use serde_json::Value;

pub fn ask_user_choice_definition() -> Value {
    super::tool_definitions::tool_def(
        "ask_user_choice",
        "Ask the user to choose between concrete options when their answer changes the next step. \
         When to use: multiple valid implementation approaches and the choice is the user's to make; ambiguous requirement where the next action depends on their preference; in Plan mode, before publishing a plan, to resolve open design questions. \
         When NOT to use: the choice has a sensible default — pick it, mention it, and proceed; you only need plan approval — use planmode, not this tool; simple clarification that you can resolve by reading the code or docs. \
         Keep it short: 1-5 questions, 2-4 options each, very short headers (max 30 chars). Every question must mark exactly one option as recommended with recommended: true — never zero or more than one. Never add recommendation wording to an option label; the UI renders the badge. The user can always pick 'Other' to type a custom answer.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "questions": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 5,
                    "items": {
                        "type": "object",
                        "properties": {
                            "header": {"type": "string", "description": "Very short label, max 30 characters"},
                            "question": {"type": "string", "description": "Question shown to the user"},
                            "multi_select": {"type": "boolean", "description": "Allow multiple options"},
                            "options": {
                                "type": "array",
                                "minItems": 2,
                                "maxItems": 4,
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {"type": "string", "description": "Stable option id for machine-readable answers"},
                                        "label": {"type": "string"},
                                        "description": {"type": "string"},
                                        "recommended": {
                                            "type": "boolean",
                                            "description": "Exactly one option in every question must set this to true. Use false or omit it for all other options. Never write recommendation text into the label."
                                        },
                                        "preview": {"type": "string"}
                                    },
                                    "required": ["label", "description"]
                                }
                            }
                        },
                        "required": ["header", "question", "options"]
                    }
                }
            },
            "required": ["questions"]
        }),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn definition_requires_one_recommendation_per_question() {
        let definition = super::ask_user_choice_definition();
        let description = definition["function"]["description"]
            .as_str()
            .unwrap_or_default();
        let recommended = definition["function"]["parameters"]["properties"]["questions"]["items"]
            ["properties"]["options"]["items"]["properties"]["recommended"]["description"]
            .as_str()
            .unwrap_or_default();

        assert!(description.contains("Every question"));
        assert!(description.contains("exactly one option"));
        assert!(description.contains("Never add recommendation wording"));
        assert!(recommended.contains("Exactly one option in every question"));
        assert!(recommended.contains("Never write recommendation text"));
    }
}
