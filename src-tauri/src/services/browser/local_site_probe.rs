use super::{
    local_site_candidates::LocalSiteCandidate,
    local_site_policy::{
        clean_local_title, is_allowed_local_url, is_internal_page, looks_like_html,
    },
    local_site_types::{LocalSite, LocalSiteProtocol, MAX_LOCAL_BODY_BYTES},
};
use futures_util::StreamExt;
use reqwest::{header::CONTENT_TYPE, redirect::Policy, Client, Response};
use std::{net::SocketAddr, time::Duration};
use zeroize::Zeroizing;

use crate::services::brand::DISPLAY_NAME;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(350);
const REQUEST_TIMEOUT: Duration = Duration::from_millis(900);
const MAX_LOCAL_REDIRECTS: usize = 3;

pub(super) async fn probe_candidate(candidate: LocalSiteCandidate) -> Result<LocalSite, ()> {
    for address in candidate.socket_addresses() {
        for protocol in [LocalSiteProtocol::Http, LocalSiteProtocol::Https] {
            if let Ok(site) = probe_endpoint(candidate.port, address, protocol).await {
                return Ok(site);
            }
        }
    }
    Err(())
}

async fn probe_endpoint(
    port: u16,
    address: SocketAddr,
    protocol: LocalSiteProtocol,
) -> Result<LocalSite, ()> {
    let client = local_client(address)?;
    let url = format!("{}://localhost:{port}/", protocol.as_str());
    let response = client
        .get(&url)
        .header("User-Agent", format!("{DISPLAY_NAME} Local Site Detector"))
        .send()
        .await
        .map_err(|_| ())?;
    if !response.status().is_success() || !is_allowed_local_url(response.url()) {
        return Err(());
    }
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let body = read_bounded_body(response).await?;
    if !looks_like_html(content_type.as_deref(), body.as_str()) || is_internal_page(body.as_str()) {
        return Err(());
    }
    let title = crate::services::link_preview::parse::extract_tag(body.as_str(), "title");
    Ok(LocalSite {
        url,
        title: clean_local_title(title, port),
        port,
        protocol,
    })
}

fn local_client(address: SocketAddr) -> Result<Client, ()> {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::custom(|attempt| {
            if attempt.previous().len() >= MAX_LOCAL_REDIRECTS
                || !is_allowed_local_url(attempt.url())
            {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .resolve("localhost", address)
        .build()
        .map_err(|_| ())
}

async fn read_bounded_body(response: Response) -> Result<Zeroizing<String>, ()> {
    let mut bytes = Zeroizing::new(Vec::with_capacity(MAX_LOCAL_BODY_BYTES.min(8 * 1024)));
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| ())?;
        let remaining = MAX_LOCAL_BODY_BYTES.saturating_sub(bytes.len());
        if remaining == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
    }
    Ok(Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()))
}
