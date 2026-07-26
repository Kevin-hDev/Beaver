use super::ollama_tool_role::{wrap_tool_results, TOOL_RESPONSE_CLOSE, TOOL_RESPONSE_OPEN};
use super::types_ollama::ChatMessage;

fn tool_msg(name: Option<&str>, content: &str) -> ChatMessage {
    ChatMessage {
        role: "tool".to_string(),
        content: content.to_string(),
        images: None,
        tool_calls: None,
        tool_name: name.map(|s| s.to_string()),
        tool_call_id: None,
        reasoning_content: None,
    }
}

fn user_msg(content: &str) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: content.to_string(),
        images: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        reasoning_content: None,
    }
}

#[test]
fn wraps_tool_message_as_user_with_tool_response() {
    let msgs = vec![tool_msg(Some("load_skill"), "Skill chargé OK")];
    let out = wrap_tool_results(&msgs);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].role, "user");
    assert!(
        out[0].content.contains(TOOL_RESPONSE_OPEN),
        "dout contenir la balise ouvrante"
    );
    assert!(
        out[0].content.contains(TOOL_RESPONSE_CLOSE),
        "doit contenir la balise fermante"
    );
    assert!(
        out[0].content.contains("Skill chargé OK"),
        "doit contenir le contenu original"
    );
}

#[test]
fn leaves_user_and_assistant_messages_untouched() {
    let msgs = vec![
        user_msg("Bonjour"),
        ChatMessage {
            role: "assistant".to_string(),
            content: "Salut".to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        },
    ];
    let out = wrap_tool_results(&msgs);
    assert_eq!(out[0].role, "user");
    assert_eq!(out[0].content, "Bonjour");
    assert_eq!(out[1].role, "assistant");
    assert_eq!(out[1].content, "Salut");
}

#[test]
fn preserves_tool_name_in_attribute() {
    let msgs = vec![tool_msg(Some("web_search"), "3 résultats trouvés")];
    let out = wrap_tool_results(&msgs);
    assert!(
        out[0].content.contains("name=\"web_search\""),
        "le tool_name doit apparaître dans un attribut : {}",
        out[0].content
    );
}

#[test]
fn empty_tool_content_still_wrapped() {
    let msgs = vec![tool_msg(Some("bash"), "")];
    let out = wrap_tool_results(&msgs);
    assert_eq!(out[0].role, "user");
    assert!(out[0].content.contains(TOOL_RESPONSE_OPEN));
    assert!(out[0].content.contains(TOOL_RESPONSE_CLOSE));
}

#[test]
fn does_not_mutate_input() {
    let msgs = vec![tool_msg(Some("grep"), "résultat")];
    let _ = wrap_tool_results(&msgs);
    // Le Vec source doit rester inchangé
    assert_eq!(msgs[0].role, "tool");
    assert_eq!(msgs[0].content, "résultat");
    assert_eq!(msgs[0].tool_name.as_deref(), Some("grep"));
}

#[test]
fn multiple_tool_messages_all_transformed() {
    let msgs = vec![
        user_msg("Charge le skill"),
        tool_msg(Some("load_skill"), "skill OK"),
        tool_msg(Some("web_search"), "3 résultats"),
        user_msg("Continue"),
    ];
    let out = wrap_tool_results(&msgs);
    assert_eq!(out.len(), 4);
    assert_eq!(out[0].role, "user");
    assert_eq!(out[1].role, "user");
    assert!(out[1].content.contains(TOOL_RESPONSE_OPEN));
    assert_eq!(out[2].role, "user");
    assert!(out[2].content.contains(TOOL_RESPONSE_OPEN));
    assert_eq!(out[3].role, "user");
    assert_eq!(out[3].content, "Continue");
}

#[test]
fn no_tool_name_omits_attribute() {
    let msgs = vec![tool_msg(None, "résultat sans nom")];
    let out = wrap_tool_results(&msgs);
    assert!(!out[0].content.contains("name="));
}
