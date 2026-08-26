pub(crate) async fn new_turn_with_writer<W, Fut>(
    session_id: &str,
    input: ResolvedTurnInput,
    target: ReplayTarget,
    writer: W,
) -> Result<AdmittedTurn, ConversationAdmissionError>
where
    W: FnOnce(AgentSession) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let _lease = super::session_locks::acquire_admission_lease(session_id).await;
    new_turn_inner(
        session_id,
        input,
        crate::services::reasoning_continuity::contract::ContinuationTarget::Replay(target),
        super::conversation_history_resolve::AttachmentKeySource::Vault,
        || async {},
        writer,
        || async {},
    )
    .await
}

pub(crate) async fn new_turn_with_after_persist<P, Fut>(
    session_id: &str,
    input: ResolvedTurnInput,
    target: ReplayTarget,
    after_persist: P,
) -> Result<AdmittedTurn, ConversationAdmissionError>
where
    P: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let _lease = super::session_locks::acquire_admission_lease(session_id).await;
    new_turn_inner(
        session_id,
        input,
        crate::services::reasoning_continuity::contract::ContinuationTarget::Replay(target),
        super::conversation_history_resolve::AttachmentKeySource::Vault,
        || async {},
        |session| async move { super::session_store::save(&session).await },
        after_persist,
    )
    .await
}

pub(crate) async fn new_turn_with_after_load<A, Fut>(
    session_id: &str,
    input: ResolvedTurnInput,
    target: ReplayTarget,
    after_load: A,
) -> Result<AdmittedTurn, ConversationAdmissionError>
where
    A: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let _lease = super::session_locks::acquire_admission_lease(session_id).await;
    new_turn_inner(
        session_id,
        input,
        crate::services::reasoning_continuity::contract::ContinuationTarget::Replay(target),
        super::conversation_history_resolve::AttachmentKeySource::Vault,
        after_load,
        |session| async move { super::session_store::save(&session).await },
        || async {},
    )
    .await
}

pub(crate) async fn new_turn_with_key(
    session_id: &str,
    input: ResolvedTurnInput,
    target: ReplayTarget,
    key: &[u8],
) -> Result<AdmittedTurn, ConversationAdmissionError> {
    let _lease = super::session_locks::acquire_admission_lease(session_id).await;
    new_turn_inner(
        session_id,
        input,
        crate::services::reasoning_continuity::contract::ContinuationTarget::Replay(target),
        super::conversation_history_resolve::AttachmentKeySource::Fixed(
            key.try_into().map_err(|_| error())?,
        ),
        || async {},
        |session| async move { super::session_store::save(&session).await },
        || async {},
    )
    .await
}

pub(crate) fn allocate_ids_for_test<F>(
    used: &mut HashSet<String>,
    generator: F,
) -> Result<(String, String, String), ConversationAdmissionError>
where
    F: FnMut() -> String,
{
    allocate_ids(used, generator)
}
