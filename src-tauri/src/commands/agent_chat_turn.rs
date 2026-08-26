use crate::models::agent_turn_contract::{ResumeTurnInput, TurnStart};
use crate::services::agent_local::conversation_admission::{
    AdmittedTurn, ConversationAdmissionError,
};
use crate::services::agent_local::conversation_input::{ConversationInputError, ResolvedTurnInput};
use crate::services::reasoning_continuity::contract::ReplayTarget;

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

pub(crate) async fn admit(
    session_id: &str,
    turn: PreparedTurn,
    target: ReplayTarget,
) -> Result<AdmittedTurn, String> {
    match turn {
        PreparedTurn::New(input) => {
            crate::services::agent_local::conversation_admission::new_turn(
                session_id, input, target,
            )
            .await
        }
        PreparedTurn::Resume(input) => {
            crate::services::agent_local::conversation_admission::resume(session_id, input, target)
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
