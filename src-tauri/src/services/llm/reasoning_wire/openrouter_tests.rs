use super::{ReasoningCapture, ReasoningCaptureContext};
use crate::services::reasoning_continuity::{
    contract::{CredentialScope, ReasoningModeId, RouteId},
    envelope::ContinuationState,
    limits::MAX_ENVELOPE_BYTES,
};
use serde_json::{json, Value};

fn capture() -> ReasoningCapture {
    ReasoningCapture::new(ReasoningCaptureContext {
        route_id: RouteId::OpenRouter,
        model_id: "z-ai/glm-5.3-flash".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::High,
    })
    .unwrap()
}

fn delta(capture: &mut ReasoningCapture, detail: Value) {
    capture.observe_json(&json!({"choices":[{"delta":{"reasoning_details":[detail]}}]}));
}

fn finish(mut capture: ReasoningCapture) -> Vec<Value> {
    capture.observe_done(&json!({"choices":[{"finish_reason":"stop"}]}));
    let envelope = capture.finish_complete().expect("complete logical blocks");
    let ContinuationState::OpenRouterDetails { details } = envelope.continuation else {
        panic!("wrong contract");
    };
    details
}

#[test]
fn streamed_text_and_summary_fragments_count_logical_blocks_not_chunks() {
    for (kind, field) in [("reasoning.text", "text"), ("reasoning.summary", "summary")] {
        let mut capture = capture();
        for _ in 0..200 {
            delta(
                &mut capture,
                json!({"type":kind,"index":0,"format":"unknown",field:"é\n"}),
            );
        }
        assert_eq!(
            finish(capture),
            vec![json!({"type":kind,"index":0,"format":"unknown",field:"é\n".repeat(200)})]
        );
    }
}

#[test]
fn late_signature_is_preserved_and_encrypted_blocks_are_never_combined() {
    let mut capture = capture();
    delta(
        &mut capture,
        json!({"type":"reasoning.text","index":0,"text":"alpha"}),
    );
    delta(
        &mut capture,
        json!({"type":"reasoning.text","index":0,"text":"β","signature":"synthetic-signature"}),
    );
    let encrypted = json!({"type":"reasoning.encrypted","index":0,"data":"synthetic-opaque"});
    delta(&mut capture, encrypted.clone());
    delta(&mut capture, encrypted.clone());
    assert_eq!(
        finish(capture),
        vec![
            json!({"type":"reasoning.text","index":0,"text":"alphaβ","signature":"synthetic-signature"}),
            encrypted.clone(),
            encrypted
        ]
    );
}

#[test]
fn different_blocks_unknown_metadata_and_complete_messages_keep_their_order() {
    let items = vec![
        json!({"type":"reasoning.text","index":0,"text":"a"}),
        json!({"type":"reasoning.text","index":1,"text":"b"}),
        json!({"type":"reasoning.text","index":1,"text":"c","future":"opaque"}),
        json!({"type":"reasoning.summary","index":1,"summary":"d"}),
    ];
    let mut streamed = capture();
    for item in &items {
        delta(&mut streamed, item.clone());
    }
    assert_eq!(finish(streamed), items);
    let mut complete = capture();
    let items = vec![json!({"type":"reasoning.text","text":"a"}); 2];
    complete.observe_json(&json!({"choices":[{"message":{"reasoning_details":items}}]}));
    assert_eq!(finish(complete), items);
}

#[test]
fn merged_fragments_still_close_capture_on_byte_limit() {
    let mut capture = capture();
    delta(&mut capture, json!({"type":"reasoning.text","text":"x"}));
    delta(
        &mut capture,
        json!({"type":"reasoning.text","text":"x".repeat(MAX_ENVELOPE_BYTES)}),
    );
    assert!(capture.is_partial());
    delta(
        &mut capture,
        json!({"type":"reasoning.text","text":"small"}),
    );
    assert!(capture.finish_complete().is_none());
}

#[test]
fn conflicts_and_unknown_shapes_are_preserved_without_signature_comparison() {
    for key in ["id", "index", "format", "signature", "summary"] {
        let mut capture = capture();
        let a = json!({"type":"reasoning.text","text":"a",key:"first"});
        let b = json!({"type":"reasoning.text","text":"b",key:"second"});
        delta(&mut capture, a.clone());
        delta(&mut capture, b.clone());
        assert_eq!(finish(capture), vec![a, b]);
    }
}

#[test]
fn signature_only_delta_does_not_erase_text_or_metadata() {
    let mut capture = capture();
    delta(
        &mut capture,
        json!({"type":"reasoning.text","text":"kept","signature":null}),
    );
    delta(
        &mut capture,
        json!({"type":"reasoning.text","signature":"synthetic","format":"anthropic-claude-v1"}),
    );
    assert_eq!(
        finish(capture),
        vec![
            json!({"type":"reasoning.text","text":"kept","signature":"synthetic","format":"anthropic-claude-v1"})
        ]
    );
}
