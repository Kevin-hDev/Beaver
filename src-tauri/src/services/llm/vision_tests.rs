use super::*;

fn user(images: Vec<&str>) -> ChatMessage {
    ChatMessage {
        role: "user".into(),
        content: "image".into(),
        images: Some(images.into_iter().map(str::to_string).collect()),
        ..Default::default()
    }
}

#[test]
fn removes_images_when_model_has_no_vision() {
    let mut messages = vec![user(vec!["iVBORw0KGgo="])];
    let report = sanitize_messages(&mut messages, false);
    assert_eq!(report.unsupported_removed, 1);
    assert!(messages[0].images.is_none());
}

#[test]
fn keeps_supported_signatures_and_limits_count() {
    let mut images = vec!["iVBORw0KGgo="; MAX_IMAGES_PER_MESSAGE + 2];
    images.push("not-base64-image");
    let mut messages = vec![user(images)];
    let report = sanitize_messages(&mut messages, true);
    assert_eq!(
        messages[0].images.as_ref().unwrap().len(),
        MAX_IMAGES_PER_MESSAGE
    );
    assert_eq!(report.invalid_removed, 3);
}

#[test]
fn builds_data_url_from_base64() {
    assert_eq!(
        data_url("iVBORw0KGgo="),
        "data:image/png;base64,iVBORw0KGgo="
    );
}
