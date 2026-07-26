use super::{
    local_site_candidates::LocalSiteCandidate, local_site_probe::probe_candidate,
    local_site_types::LocalSiteProtocol,
};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn probes_a_bounded_local_html_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/html")
                .set_body_string("<html><title>Projet local</title><body>ok</body></html>"),
        )
        .mount(&server)
        .await;

    let site = probe_candidate(LocalSiteCandidate {
        port: server.address().port(),
        ipv4: true,
        ipv6: false,
    })
    .await
    .unwrap();

    assert_eq!(site.title, "Projet local");
    assert_eq!(site.protocol, LocalSiteProtocol::Http);
    assert_eq!(site.url, format!("http://localhost:{}/", site.port));
}

#[tokio::test]
async fn ignores_non_html_responses() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string("{\"status\":\"ok\"}"),
        )
        .mount(&server)
        .await;

    let result = probe_candidate(LocalSiteCandidate {
        port: server.address().port(),
        ipv4: true,
        ipv6: false,
    })
    .await;
    assert!(result.is_err());
}
