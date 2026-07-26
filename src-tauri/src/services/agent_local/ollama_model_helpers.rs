use crate::services::agent_local::types_ollama::{ModelInfo, OllamaModel};

pub(crate) fn dedupe_by_digest(models: Vec<OllamaModel>) -> Vec<OllamaModel> {
    use std::collections::HashMap;
    let mut groups: HashMap<String, Vec<OllamaModel>> = HashMap::new();
    let mut no_digest: Vec<OllamaModel> = Vec::new();
    for m in models {
        if m.digest_short.is_empty() {
            no_digest.push(m);
        } else {
            groups.entry(m.digest_short.clone()).or_default().push(m);
        }
    }
    let mut result: Vec<OllamaModel> = Vec::new();
    for (_digest, mut group) in groups {
        group.sort_by(|a, b| {
            let al = a.name.ends_with(":latest");
            let bl = b.name.ends_with(":latest");
            al.cmp(&bl).then_with(|| a.name.len().cmp(&b.name.len()))
        });
        let mut primary = group.remove(0);
        primary.aliases = group.into_iter().map(|m| m.name).collect();
        result.push(primary);
    }
    result.extend(no_digest);
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

pub(crate) fn build_model_from_tags(
    m: &serde_json::Value,
    info: Option<ModelInfo>,
    is_customized: bool,
) -> OllamaModel {
    let name = m["name"].as_str().unwrap_or_default().to_string();
    let details = &m["details"];
    let digest_short: String = m["digest"]
        .as_str()
        .unwrap_or_default()
        .trim_start_matches("sha256:")
        .chars()
        .take(12)
        .collect();
    OllamaModel {
        name,
        size: m["size"].as_u64().unwrap_or(0),
        family: info
            .as_ref()
            .map_or_else(|| s(details, "family"), |i| i.family.clone()),
        parameter_size: info.as_ref().map_or_else(
            || s(details, "parameter_size"),
            |i| i.parameter_size.clone(),
        ),
        quantization: info.as_ref().map_or_else(
            || s(details, "quantization_level"),
            |i| i.quantization.clone(),
        ),
        architecture: info
            .as_ref()
            .map_or_else(String::new, |i| i.architecture.clone()),
        is_moe: info.as_ref().is_some_and(|i| i.is_moe),
        context_length: info.as_ref().map_or(0, |i| i.context_length),
        capabilities: info.map_or_else(|| vec!["completion".to_string()], |i| i.capabilities),
        digest_short,
        aliases: Vec::new(),
        is_customized,
    }
}

pub(crate) fn parse_show_response(name: &str, json: &serde_json::Value) -> ModelInfo {
    let details = &json["details"];
    let mi = &json["model_info"];
    let arch = mi["general.architecture"].as_str().unwrap_or("");

    ModelInfo {
        name: name.to_string(),
        modelfile: s(json, "modelfile"),
        parameters: s(json, "parameters"),
        template: s(json, "template"),
        family: s(details, "family"),
        parameter_size: s(details, "parameter_size"),
        quantization: s(details, "quantization_level"),
        architecture: arch.to_string(),
        is_moe: mi
            .get(format!("{arch}.expert_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        context_length: mi[format!("{arch}.context_length")]
            .as_u64()
            .unwrap_or(4096),
        capabilities: parse_capabilities(json),
        has_audio: json["capabilities"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("audio"))),
        license: s(json, "license"),
    }
}

fn parse_capabilities(json: &serde_json::Value) -> Vec<String> {
    json["capabilities"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_else(|| vec!["completion".to_string()])
}

fn s(v: &serde_json::Value, key: &str) -> String {
    v[key].as_str().unwrap_or("").to_string()
}
