use super::{to_tool_result, HostToolResult};
use crate::services::agent_local::tool_artifact::ArtifactPurpose;
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::extensions::tool_result_contract::{ToolResultBlock, ToolResultContent};

fn host_result(content: ToolResultContent) -> HostToolResult {
    HostToolResult {
        content,
        is_error: false,
        truncated: false,
        display_summary: None,
    }
}

#[test]
fn historical_string_content_is_preserved_byte_for_byte() {
    let content = "historique\0\n".to_string();
    let result = to_tool_result(Ok(host_result(ToolResultContent::Text(content.clone()))));

    assert_eq!(result.content.as_bytes(), content.as_bytes());
    assert!(result.pending_artifacts().is_empty());
}

#[test]
fn rich_text_only_keeps_its_text() {
    let result = to_tool_result(Ok(host_result(ToolResultContent::Blocks(vec![
        ToolResultBlock::Text {
            text: "texte seul".to_string(),
        },
    ]))));

    assert_eq!(result.content, "texte seul");
    assert!(result.pending_artifacts().is_empty());
}

#[test]
fn rich_file_becomes_a_pending_artifact_without_a_read() {
    let result = to_tool_result(Ok(host_result(ToolResultContent::Blocks(vec![
        ToolResultBlock::File {
            path: "report.csv".to_string(),
            purpose: ArtifactPurpose::Artifact,
            display_name: None,
        },
    ]))));

    assert_eq!(result.content, "[artifact: report.csv]");
    let pending = result.pending_artifacts();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].relative_path, "report.csv");
    assert_eq!(pending[0].display_name, None);
    assert_eq!(pending[0].purpose, ArtifactPurpose::Artifact);
}

#[test]
fn rich_blocks_keep_their_original_order() {
    let result = to_tool_result(Ok(host_result(ToolResultContent::Blocks(vec![
        ToolResultBlock::Text {
            text: "avant".to_string(),
        },
        ToolResultBlock::File {
            path: "report.csv".to_string(),
            purpose: ArtifactPurpose::Preview,
            display_name: Some("Rapport".to_string()),
        },
        ToolResultBlock::Text {
            text: "après".to_string(),
        },
    ]))));

    assert_eq!(result.content, "avant[preview: Rapport]après");
    assert_eq!(
        result.pending_artifacts()[0].purpose,
        ArtifactPurpose::Preview
    );
    assert_eq!(
        result.pending_artifacts()[0].display_name.as_deref(),
        Some("Rapport")
    );
}

#[test]
fn extension_reported_error_with_a_file_is_refused_as_one_invalid_result() {
    let mut host_result = host_result(ToolResultContent::Blocks(vec![
        ToolResultBlock::Text {
            text: "entrée invalide".to_string(),
        },
        ToolResultBlock::File {
            path: "report.csv".to_string(),
            purpose: ArtifactPurpose::Artifact,
            display_name: None,
        },
    ]));
    host_result.is_error = true;

    let result = to_tool_result(Ok(host_result));

    assert!(result.is_error);
    assert_eq!(result.content, "Résultat d'extension indisponible.");
    assert!(result.pending_artifacts().is_empty());
    let error = result.error.expect("structured extension error");
    assert_eq!(error.category, ToolErrorCategory::Unavailable);
    assert_eq!(
        error.code.as_ref(),
        crate::services::extensions::error_codes::RESULT_INVALID
    );
}

#[test]
fn uncertain_host_failure_never_recommends_a_blind_retry() {
    let result = to_tool_result(Err("host disconnected".to_string()));
    let error = result.error.expect("structured extension error");
    assert_eq!(
        error.code.as_ref(),
        crate::services::extensions::error_codes::HOST_UNAVAILABLE
    );
    assert!(!error.retryable);
    assert!(error.hint.is_some());
}
