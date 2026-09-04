use serde_json::{Map, Value};

pub fn validate(
    limits: &Map<String, Value>,
    host_limits: &Map<String, Value>,
) -> Result<(), String> {
    let limit = |name| usize_value(limits, name, "invalid extension discovery contract limit");
    let identifier_length = usize_value(
        host_limits,
        "maxIdentifierChars",
        "invalid host extension identifier limit",
    )?;
    let tools = host_limit(host_limits, "maxToolsPerExtension")?;
    let skills = host_limit(host_limits, "maxSkillsPerExtension")?;
    let resources = host_limit(host_limits, "maxResourcesPerExtension")?;
    let extensions = host_limit(host_limits, "maxExtensions")?;
    let inspected_extensions = limit("maxInspectedExtensions")?;
    ensure_identifier_width(identifier_length, [extensions, tools, skills, resources])?;
    let extension_name = json_bounded_text(limit("maxProjectedExtensionNameJsonBytes")?);
    let extension_description =
        json_bounded_text(limit("maxProjectedExtensionDescriptionJsonBytes")?);
    let contribution_name = json_bounded_text(limit("maxProjectedContributionNameJsonBytes")?);
    let contribution_summary =
        json_bounded_text(limit("maxProjectedContributionSummaryJsonBytes")?);
    ensure_count_is_bounded(extensions, limit("maxCompactCatalogBytes")?, 19)?;
    let contributions = tools
        .checked_add(skills)
        .and_then(|value| value.checked_add(resources))
        .and_then(|value| value.checked_mul(inspected_extensions))
        .ok_or_else(|| "extension discovery proof count exceeds its limit".to_string())?;
    ensure_count_is_bounded(contributions, limit("maxSerializedResultBytes")?, 44)?;
    ensure_identifier_bytes(
        extensions,
        identifier_length,
        limit("maxCompactCatalogBytes")?,
    )?;
    let qualified_identifier_length = identifier_length
        .checked_mul(2)
        .and_then(|value| value.checked_add("extension::".len()))
        .ok_or_else(|| "extension discovery identifier budget exceeds its limit".to_string())?;
    let public_identifier_bytes = tools
        .checked_mul(identifier_length)
        .and_then(|value| {
            skills
                .checked_add(resources)
                .and_then(|count| count.checked_mul(qualified_identifier_length))
                .and_then(|qualified| value.checked_add(qualified))
        })
        .and_then(|value| value.checked_mul(inspected_extensions))
        .ok_or_else(|| "extension discovery identifier budget exceeds its limit".to_string())?;
    ensure_identifier_bytes(
        public_identifier_bytes,
        1,
        limit("maxSerializedResultBytes")?,
    )?;
    let compact = (0..extensions)
        .map(|index| {
            Ok(serde_json::json!({
                "id": canonical_identifier(identifier_length, index)?,
                "name": extension_name,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    fits_json(
        &compact,
        limit("maxCompactCatalogBytes")?,
        "compact catalog",
    )?;

    let contribution = |kind: &str, index, qualified: bool, extension_index| {
        let id = if qualified {
            qualified_identifier(identifier_length, extension_index, index)?
        } else {
            canonical_identifier(identifier_length, index)?
        };
        Ok(serde_json::json!({
            "id": id,
            "name": contribution_name,
            "summary": contribution_summary,
            "type": kind,
        }))
    };
    let inspected = (0..inspected_extensions)
        .map(|index| {
            Ok(serde_json::json!({
                "id": canonical_identifier(identifier_length, index)?,
                "name": extension_name,
                "description": extension_description,
                "status": "limited_by_provider",
                "tools": (0..tools)
                    .map(|item| contribution("tool", item, false, index))
                    .collect::<Result<Vec<_>, String>>()?,
                "skills": (0..skills)
                    .map(|item| contribution("skill", item, true, index))
                    .collect::<Result<Vec<_>, String>>()?,
                "resources": (0..resources)
                    .map(|item| contribution("resource", item, true, index))
                    .collect::<Result<Vec<_>, String>>()?,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    fits_json(
        &inspected,
        limit("maxSerializedResultBytes")?,
        "serialized result",
    )
}

fn host_limit(limits: &Map<String, Value>, name: &str) -> Result<usize, String> {
    usize_value(limits, name, "invalid imported extension limit")
}

fn usize_value(limits: &Map<String, Value>, name: &str, error: &str) -> Result<usize, String> {
    limits
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| error.to_string())
}

fn ensure_count_is_bounded(
    count: usize,
    budget: usize,
    minimum_bytes: usize,
) -> Result<(), String> {
    count
        .checked_mul(minimum_bytes)
        .is_some_and(|bytes| bytes <= budget)
        .then_some(())
        .ok_or_else(|| "extension discovery proof count exceeds its limit".to_string())
}

fn ensure_identifier_bytes(count: usize, width: usize, budget: usize) -> Result<(), String> {
    count
        .checked_mul(width)
        .is_some_and(|bytes| bytes <= budget)
        .then_some(())
        .ok_or_else(|| "extension discovery identifier budget exceeds its limit".to_string())
}

fn ensure_identifier_width(maximum: usize, counts: [usize; 4]) -> Result<(), String> {
    let largest_index = counts.into_iter().max().unwrap_or(0).saturating_sub(1);
    let mut suffix_chars = 1;
    let mut remaining = largest_index;
    while remaining >= 10 {
        remaining /= 10;
        suffix_chars += 1;
    }
    (maximum >= suffix_chars + 1)
        .then_some(())
        .ok_or_else(|| "extension discovery identifier limit is too small".to_string())
}

fn fits_json(value: &[Value], maximum: usize, subject: &str) -> Result<(), String> {
    (serde_json::to_vec(value)
        .map_err(|_| "cannot serialize extension discovery proof".to_string())?
        .len()
        <= maximum)
        .then_some(())
        .ok_or_else(|| format!("extension discovery {subject} exceeds its limit"))
}

fn json_bounded_text(maximum: usize) -> String {
    let mut output = String::new();
    for character in "🦫\\\"\n\u{0001}".chars().cycle() {
        let mut candidate = output.clone();
        candidate.push(character);
        if serde_json::to_vec(&candidate).is_ok_and(|value| value.len() > maximum) {
            return output;
        }
        output = candidate;
    }
    unreachable!()
}

fn canonical_identifier(maximum: usize, index: usize) -> Result<String, String> {
    let suffix = index.to_string();
    let padding = maximum
        .checked_sub(suffix.len() + 1)
        .ok_or_else(|| "extension discovery identifier limit is too small".to_string())?;
    Ok(format!("a{}{}", "a".repeat(padding), suffix))
}

fn qualified_identifier(maximum: usize, extension: usize, local: usize) -> Result<String, String> {
    Ok(format!(
        "extension:{}:{}",
        canonical_identifier(maximum, extension)?,
        canonical_identifier(maximum, local)?,
    ))
}
