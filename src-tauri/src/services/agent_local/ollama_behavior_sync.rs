pub fn system_prompt_after_modelfile_edit(
    previous_override: Option<&str>,
    current_modelfile: &str,
    updated_modelfile: &str,
) -> Option<String> {
    let current_system = parse_system(current_modelfile);
    let updated_system = parse_system(updated_modelfile);
    if previous_override.is_some() || current_system != updated_system {
        Some(updated_system)
    } else {
        None
    }
}

fn parse_system(content: &str) -> String {
    super::modelfile_parser::parse_modelfile(content)
        .system
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrelated_raw_edit_does_not_activate_a_baked_system_prompt() {
        let current = "FROM base\nSYSTEM baked\nPARAMETER temperature 0.7";
        let updated = "FROM base\nSYSTEM baked\nPARAMETER temperature 0.4";

        assert_eq!(system_prompt_after_modelfile_edit(None, current, updated), None);
    }

    #[test]
    fn raw_system_change_becomes_the_behavior_override() {
        let current = "FROM base\nSYSTEM old";
        let updated = "FROM base\nSYSTEM new";

        assert_eq!(
            system_prompt_after_modelfile_edit(None, current, updated),
            Some("new".to_string())
        );
    }

    #[test]
    fn removing_raw_system_clears_an_existing_behavior_override() {
        let current = "FROM base\nSYSTEM custom";
        let updated = "FROM base";

        assert_eq!(
            system_prompt_after_modelfile_edit(Some("custom"), current, updated),
            Some(String::new())
        );
    }
}
