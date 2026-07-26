use super::model_details::ForecastModelDetails;
use super::{model_details_http, model_details_markdown};
use regex::Regex;
use std::sync::LazyLock;

static HF_LINK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\]\(([^)]+)\)"#).unwrap());
static HF_RELATIVE_IMAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"!\[([^\]]*)\]\(([^)]+)\)"#).unwrap());

pub async fn fetch(repo: &str, revision: Option<&str>) -> Result<ForecastModelDetails, String> {
    let client = model_details_http::client()?;
    let meta = model_details_http::metadata(
        &client,
        &format!("https://huggingface.co/api/models/{repo}"),
    )
    .await?;
    let rev = revision.unwrap_or("main");
    let markdown = model_details_http::optional_markdown(
        &client,
        &format!("https://huggingface.co/{repo}/raw/{rev}/README.md"),
    )
    .await;
    let card = meta.get("cardData").and_then(|value| value.as_object());

    Ok(ForecastModelDetails {
        description_short: short_description(card),
        description_long_markdown: absolutize(repo, rev, strip_frontmatter(markdown)),
        source_url: format!("https://huggingface.co/{repo}"),
        source_label: "Hugging Face".to_string(),
        license: card_value(card, "license"),
        pipeline_tag: string_value(&meta, "pipeline_tag"),
        library_name: string_value(&meta, "library_name"),
        downloads: meta.get("downloads").and_then(|value| value.as_u64()),
        likes: meta.get("likes").and_then(|value| value.as_u64()),
        tags: tags(&meta),
    })
}

fn short_description(card: Option<&serde_json::Map<String, serde_json::Value>>) -> String {
    card.and_then(|values| values.get("model_summary"))
        .and_then(|value| value.as_str())
        .or_else(|| {
            card.and_then(|values| values.get("description"))
                .and_then(|value| value.as_str())
        })
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn card_value(
    card: Option<&serde_json::Map<String, serde_json::Value>>,
    key: &str,
) -> Option<String> {
    card.and_then(|values| values.get(key))
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn string_value(meta: &serde_json::Value, key: &str) -> Option<String> {
    meta.get(key)
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn tags(meta: &serde_json::Value) -> Vec<String> {
    meta.get("tags")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str())
                .filter(|tag| !tag.starts_with("region:"))
                .take(16)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn absolutize(repo: &str, revision: &str, markdown: String) -> String {
    let base_blob = format!("https://huggingface.co/{repo}/blob/{revision}/");
    let base_resolve = format!("https://huggingface.co/{repo}/resolve/{revision}/");
    let images = HF_RELATIVE_IMAGE.replace_all(&markdown, |caps: &regex::Captures| {
        let target = &caps[2];
        if model_details_markdown::is_relative_target(target) {
            format!(
                "![{}]({}{})",
                &caps[1],
                base_resolve,
                model_details_markdown::normalize_relative_target(target)
            )
        } else {
            caps[0].to_string()
        }
    });
    HF_LINK
        .replace_all(&images, |caps: &regex::Captures| {
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

fn strip_frontmatter(markdown: String) -> String {
    if let Some(rest) = markdown.strip_prefix("---\n") {
        let mut parts = rest.splitn(2, "\n---\n");
        let _frontmatter = parts.next();
        if let Some(content) = parts.next() {
            return content.trim_start().to_string();
        }
    }
    markdown
}
