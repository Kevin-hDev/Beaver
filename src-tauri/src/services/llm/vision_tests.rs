use super::*;

fn user(images: Vec<&str>) -> ChatMessage {
    ChatMessage::user("image".into()).with_images(images.into_iter().map(str::to_string).collect())
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

#[test]
fn vision_wire_formats_stay_distinct() {
    let message = user(vec!["iVBORw0KGgo="]);
    let openai_policy =
        super::super::route_profile::payload_policy("google", "gemini-3.5-flash").unwrap();
    let mistral_policy =
        super::super::route_profile::payload_policy("mistral", "mistral-large").unwrap();
    let openai =
        crate::services::llm::stream_convert::message_to_openai(&message, openai_policy.message);
    let mistral =
        crate::services::llm::stream_convert::message_to_openai(&message, mistral_policy.message);

    assert_eq!(
        openai["content"][1]["image_url"]["url"],
        data_url("iVBORw0KGgo=")
    );
    assert_eq!(mistral["content"][1]["image_url"], data_url("iVBORw0KGgo="));
}

#[test]
fn unsupported_image_wire_is_rejected_explicitly() {
    assert_eq!(
        image_part(
            "iVBORw0KGgo=",
            super::super::route_profile::ImageFormat::Unsupported,
        ),
        Err("vision_wire_unsupported")
    );
}
