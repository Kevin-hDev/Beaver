use serde_json::Value;

pub fn get_chat_tool_definitions() -> Vec<Value> {
    super::tool_definitions_web::web_tool_definitions()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_exposes_only_web_search_and_fetch() {
        let definitions = get_chat_tool_definitions();
        let names = definitions
            .iter()
            .filter_map(|tool| tool.pointer("/function/name").and_then(Value::as_str))
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["web_search", "web_fetch"]);
    }
}
