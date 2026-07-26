use super::model_details::ForecastModelDetails;
use super::{model_details_http, model_details_markdown};
use regex::Regex;
use std::sync::LazyLock;

static GH_LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\]\(([^)]+)\)"#).unwrap());
static GH_RELATIVE_IMAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"!\[([^\]]*)\]\(([^)]+)\)"#).unwrap());
static GH_HTML_SRC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"src="([^"]+)""#).unwrap());

pub async fn fetch_github(
    repo: &str,
    revision: Option<&str>,
) -> Result<ForecastModelDetails, String> {
    let client = model_details_http::client()?;
    let meta =
        model_details_http::metadata(&client, &format!("https://api.github.com/repos/{repo}"))
            .await?;
    let rev = revision.unwrap_or("main");
    let markdown = model_details_http::optional_markdown(
        &client,
        &format!("https://raw.githubusercontent.com/{repo}/{rev}/README.md"),
    )
    .await;

    Ok(ForecastModelDetails {
        description_short: meta
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        description_long_markdown: absolutize_github_markdown(repo, rev, markdown),
        source_url: format!("https://github.com/{repo}"),
        source_label: "GitHub".to_string(),
        license: meta
            .get("license")
            .and_then(|v| v.get("spdx_id"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        pipeline_tag: None,
        library_name: None,
        downloads: None,
        likes: meta.get("stargazers_count").and_then(|v| v.as_u64()),
        tags: Vec::new(),
    })
}

fn absolutize_github_markdown(repo: &str, revision: &str, markdown: String) -> String {
    let base_blob = format!("https://github.com/{repo}/blob/{revision}/");
    let base_raw = format!("https://raw.githubusercontent.com/{repo}/{revision}/");
    let step1 = GH_RELATIVE_IMAGE.replace_all(&markdown, |caps: &regex::Captures| {
        let target = &caps[2];
        if model_details_markdown::is_relative_target(target) {
            format!(
                "![{}]({}{})",
                &caps[1],
                base_raw,
                model_details_markdown::normalize_relative_target(target)
            )
        } else {
            caps[0].to_string()
        }
    });
    let step2 = GH_HTML_SRC.replace_all(&step1, |caps: &regex::Captures| {
        let target = &caps[1];
        if model_details_markdown::is_relative_target(target) {
            format!(
                "src=\"{}{}\"",
                base_raw,
                model_details_markdown::normalize_relative_target(target)
            )
        } else {
            caps[0].to_string()
        }
    });
    GH_LINK
        .replace_all(&step2, |caps: &regex::Captures| {
            let target = &caps[1];
            if model_details_markdown::is_relative_target(target) {
                format!(
                    "]({}{})",
                    base_blob,
                    model_details_markdown::normalize_relative_target(target)
                )
            } else {
                caps[0].to_string()
            }
        })
        .into_owned()
}
