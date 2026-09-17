pub(crate) fn response_language_instruction(lang: &str) -> Option<String> {
    (!lang.is_empty()).then(|| {
        format!(
            "\n\nYou MUST respond in {lang}. All your answers, explanations and communications must be in {lang}."
        )
    })
}

pub(crate) fn skills_listing_section(skills: &[(String, String)]) -> Option<String> {
    if skills.is_empty() {
        return None;
    }
    let listing = skills
        .iter()
        .map(|(name, desc)| format!("- {name}: {desc}"))
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!(
        "\n\n## Available skills\n\
         The following skills are available. Use the `load_skill` tool to load one when relevant.\n\
         {listing}"
    ))
}
