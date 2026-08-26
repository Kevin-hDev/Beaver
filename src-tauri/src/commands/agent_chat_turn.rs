use crate::models::agent_turn_contract::{ResumeTurnInput, TurnStart};
use crate::services::agent_local::conversation_admission::{
    AdmittedTurn, ConversationAdmissionError,
};
use crate::services::agent_local::conversation_input::{ConversationInputError, ResolvedTurnInput};
use crate::services::reasoning_continuity::contract::ContinuationTarget;

pub(crate) enum PreparedTurn {
    New(ResolvedTurnInput),
    Resume(ResumeTurnInput),
}

pub(crate) async fn prepare(turn: TurnStart) -> Result<PreparedTurn, String> {
    match turn {
        TurnStart::New(input) => crate::services::agent_local::conversation_input::resolve(input)
            .await
            .map(PreparedTurn::New)
            .map_err(public_input_error),
        TurnStart::Resume(input) => Ok(PreparedTurn::Resume(input)),
    }
}

pub(crate) async fn admit_current(
    streams: &crate::ActiveStreams,
    session_id: &str,
    generation: u64,
    turn: PreparedTurn,
    target: ContinuationTarget,
) -> Result<AdmittedTurn, String> {
    let lease = crate::services::agent_local::session_store::lock_session(session_id).await;
    let _guard = lease.lock().await;
    let current = matches!(
        streams.0.lock().await.get(session_id),
        Some((_, active, _, _)) if *active == generation
    );
    if !current {
        return Err("conversation_admission_failed".to_string());
    }
    match turn {
        PreparedTurn::New(input) => {
            crate::services::agent_local::conversation_admission::new_turn_with_lease(
                session_id, input, target,
            )
            .await
        }
        PreparedTurn::Resume(input) => {
            crate::services::agent_local::conversation_resume::resume_with_lease(
                session_id, input, target,
            )
            .await
        }
    }
    .map_err(public_admission_error)
}

fn public_input_error(_: ConversationInputError) -> String {
    "conversation_admission_failed".to_string()
}

fn public_admission_error(_: ConversationAdmissionError) -> String {
    "conversation_admission_failed".to_string()
}
