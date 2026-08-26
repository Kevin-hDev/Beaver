use crate::models::agent_turn_contract::{ResumeTurnInput, TurnStart};
use crate::services::agent_local::conversation_admission::{
    AdmittedTurn, ConversationAdmissionError,
};
use crate::services::agent_local::conversation_input::{ConversationInputError, ResolvedTurnInput};
use crate::services::reasoning_continuity::contract::ContinuationTarget;

pub(crate) struct AdmittedCurrentTurn {
    pub(crate) turn: AdmittedTurn,
    before: crate::services::agent_local::types_session::AgentSession,
    kind: AdmittedTurnKind,
}

#[derive(Clone, Copy)]
enum AdmittedTurnKind {
    New,
    Resume,
}

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
    reasoning: crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate,
) -> Result<AdmittedCurrentTurn, String> {
    let lease =
        crate::services::agent_local::session_locks::acquire_admission_lease(session_id).await;
    let current = matches!(
        streams.0.lock().await.get(session_id),
        Some((_, active, _, _)) if *active == generation
    );
    if !current {
        return Err("conversation_admission_failed".to_string());
    }
    let before = crate::services::agent_local::session_store::get(session_id)
        .await
        .map_err(|_| "conversation_admission_failed".to_string())?;
    let (turn, kind) = match turn {
        PreparedTurn::New(input) => {
            crate::services::agent_local::conversation_admission::new_turn_with_lease_and_reasoning(
                &lease, input, target, &reasoning,
            )
            .await
            .map(|turn| (turn, AdmittedTurnKind::New))
        }
        PreparedTurn::Resume(input) => {
            crate::services::agent_local::conversation_resume::resume_with_lease_and_reasoning(
                &lease, input, target, &reasoning,
            )
            .await
            .map(|turn| (turn, AdmittedTurnKind::Resume))
        }
    }
    .map_err(public_admission_error)?;
    Ok(AdmittedCurrentTurn { turn, before, kind })
}

/// Annule l'admission durable si une étape préparatoire postérieure échoue
/// avant le lancement du modèle. Le lease et la génération évitent d'écraser
/// une nouvelle requête qui aurait déjà remplacé celle-ci.
pub(crate) async fn rollback_current(
    streams: &crate::ActiveStreams,
    session_id: &str,
    generation: u64,
    admitted: &AdmittedCurrentTurn,
) -> Result<(), String> {
    let lease =
        crate::services::agent_local::session_locks::acquire_admission_lease(session_id).await;
    let current = matches!(
        streams.0.lock().await.get(session_id),
        Some((_, active, _, _)) if *active == generation
    );
    if !current {
        return Ok(());
    }

    let mut session = crate::services::agent_local::session_store::get(lease.session_id())
        .await
        .map_err(|_| "conversation_admission_failed".to_string())?;
    if !matches_admitted_turn(&session, admitted) {
        return Err("conversation_admission_failed".to_string());
    }

    restore_before_admission(&mut session, &admitted.before);
    crate::services::agent_local::session_store::save(&session)
        .await
        .map_err(|_| "conversation_admission_failed".to_string())
}

fn matches_admitted_turn(
    session: &crate::services::agent_local::types_session::AgentSession,
    admitted: &AdmittedCurrentTurn,
) -> bool {
    match admitted.kind {
        AdmittedTurnKind::New => {
            session.messages.len() == admitted.before.messages.len() + 1
                && session.messages.last().is_some_and(|message| {
                    message.id == admitted.turn.user_message_id
                        && message.turn_id == admitted.turn.turn_id
                        && message.role == "user"
                })
        }
        AdmittedTurnKind::Resume => {
            session.messages.len() == admitted.before.messages.len()
                && session.messages.last().is_some_and(|message| {
                    message.id == admitted.turn.user_message_id
                        && message.turn_id == admitted.turn.turn_id
                        && message.role == "user"
                })
        }
    }
}

fn restore_before_admission(
    session: &mut crate::services::agent_local::types_session::AgentSession,
    before: &crate::services::agent_local::types_session::AgentSession,
) {
    // Les diagnostics de requête restent écrits : ils expliquent l'échec.
    // Seules les mutations propres à l'admission du tour sont annulées.
    session.reasoning_mode.clone_from(&before.reasoning_mode);
    session.thinking_enabled = before.thinking_enabled;
    session.messages.clone_from(&before.messages);
    session.todos.clone_from(&before.todos);
    session.todo_neglect_count = before.todo_neglect_count;
    session
        .active_todo_run_id
        .clone_from(&before.active_todo_run_id);
    session.accumulated_tokens = before.accumulated_tokens;
    session.context_tokens = before.context_tokens;
    session.updated_at = before.updated_at;
}

fn public_input_error(_: ConversationInputError) -> String {
    "conversation_admission_failed".to_string()
}

fn public_admission_error(_: ConversationAdmissionError) -> String {
    "conversation_admission_failed".to_string()
}
