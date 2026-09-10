use crate::services::agent_local::conversation_input::ResolvedTurnInput;
use crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate;
use crate::services::agent_local::session_locks::AdmissionLease;
use crate::services::reasoning_continuity::contract::ContinuationTarget;

pub(crate) async fn new_turn_with_lease_and_reasoning(
    lease: &AdmissionLease,
    input: ResolvedTurnInput,
    target: ContinuationTarget,
    reasoning: &SessionReasoningUpdate,
) -> Result<super::AdmittedTurn, super::ConversationAdmissionError> {
    admitted(lease, input, target, reasoning, None).await
}

pub(crate) async fn new_automation_turn_with_lease_and_reasoning(
    lease: &AdmissionLease,
    input: ResolvedTurnInput,
    target: ContinuationTarget,
    reasoning: &SessionReasoningUpdate,
) -> Result<super::AdmittedTurn, super::ConversationAdmissionError> {
    admitted(
        lease,
        input,
        target,
        reasoning,
        Some(crate::services::agent_local::types_message::AgentMessageKind::Automation),
    )
    .await
}

async fn admitted(
    lease: &AdmissionLease,
    input: ResolvedTurnInput,
    target: ContinuationTarget,
    reasoning: &SessionReasoningUpdate,
    kind: Option<crate::services::agent_local::types_message::AgentMessageKind>,
) -> Result<super::AdmittedTurn, super::ConversationAdmissionError> {
    super::new_turn_inner(
        lease.session_id(),
        input,
        target,
        Some(reasoning),
        kind,
        crate::services::agent_local::conversation_history_resolve::AttachmentKeySource::Vault,
        || async {},
        |session| async move {
            crate::services::agent_local::session_store::save(&session).await
        },
        || async {},
    )
    .await
}
