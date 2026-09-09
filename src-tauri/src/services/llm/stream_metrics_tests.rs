use super::*;
use crate::services::provider_usage::{RequestUsage, UsageApiFormat};

#[tokio::test]
async fn failed_completion_keeps_billed_usage_and_finish_reason_in_both_readers() {
    for silent in [false, true] {
        let id = uuid::Uuid::new_v4().to_string();
        let measurement = RequestMeasurement::start(RequestMeasurementContext {
            connection_id: "openrouter",
            canonical_provider_id: "openrouter",
            api_format: UsageApiFormat::ChatCompletions,
            model: "fixture",
            session_id: None,
            request_id: &id,
            turn: None,
            attempt: 1,
            workload: UsageWorkload::Primary,
            fast_mode: super::super::fast_mode::FastModeRequest::Standard,
        })
        .unwrap();
        let result = StreamResult {
            done_reason: Some("length".into()),
            completion_error: Some("provider_output_limit"),
            usage: Some(RequestUsage {
                input_tokens: Some(739),
                output_tokens: Some(512),
                reasoning_output_tokens: Some(505),
                ..Default::default()
            }),
            ..Default::default()
        };
        if silent {
            finish_silent(Some(measurement), &Ok(result)).await;
        } else {
            finish_stream(Some(measurement), &Ok(StreamOutcome::Completed(result))).await;
        }
        let bytes = tokio::fs::read(
            crate::services::paths::data_dir().join("provider-request-metrics.json"),
        )
        .await
        .unwrap();
        let snapshot: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let saved = snapshot["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["request_id"] == id)
            .unwrap();
        assert_eq!(saved["status"], "failed");
        assert_eq!(saved["finish_reason"], "length");
        assert_eq!(saved["usage"]["output_tokens"], 512);
        assert_eq!(saved["usage"]["reasoning_output_tokens"], 505);
        assert_eq!(saved["usage_complete"], false);
    }
}
