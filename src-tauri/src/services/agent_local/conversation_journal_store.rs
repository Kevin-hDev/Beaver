use std::future::Future;
use std::pin::Pin;

use super::{error, ConversationJournal};

type SaveFuture<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

impl ConversationJournal {
    pub(crate) async fn commit_turn(&mut self) -> Result<(), String> {
        self.commit_turn_with_writer(|session| Box::pin(super::super::session_store::save(session)))
            .await
    }

    async fn commit_turn_with_writer<F>(&mut self, writer: F) -> Result<(), String>
    where
        F: for<'a> FnOnce(&'a super::super::types_session::AgentSession) -> SaveFuture<'a>,
    {
        if self.committed
            || self.partial
            || self.assistant_steps == 0
            || !self.expected_tool_ids.is_empty()
        {
            return Err(error());
        }
        let run_id = self.request_id.clone();
        self.update_with_writer(
            move |session| {
                let mut found = false;
                for message in &mut session.messages {
                    if message.stream_run_id.as_deref() == Some(&run_id) {
                        message.stream_part = Some("final".to_string());
                        found = true;
                    }
                }
                found.then_some(()).ok_or_else(error)
            },
            writer,
        )
        .await?;
        self.committed = true;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) async fn commit_turn_with_injected_write_failure(&mut self) -> Result<(), String> {
        self.commit_turn_with_writer(|_| Box::pin(async { Err(error()) }))
            .await
    }

    pub(super) async fn update<F>(&self, update: F) -> Result<(), String>
    where
        F: FnOnce(&mut super::super::types_session::AgentSession) -> Result<(), String>,
    {
        self.update_with_writer(update, |session| {
            Box::pin(super::super::session_store::save(session))
        })
        .await
    }

    async fn update_with_writer<F, W>(&self, update: F, writer: W) -> Result<(), String>
    where
        F: FnOnce(&mut super::super::types_session::AgentSession) -> Result<(), String>,
        W: for<'a> FnOnce(&'a super::super::types_session::AgentSession) -> SaveFuture<'a>,
    {
        self.verify_subagent_owner().await?;
        let lock = super::super::session_store::lock_session(&self.session_id).await;
        let _guard = lock.lock().await;
        let mut session = super::super::session_store::get(&self.session_id)
            .await
            .map_err(|_| error())?;
        if self
            .subagent_owner
            .as_ref()
            .is_some_and(|owner| session.subagent_run_id.as_deref() != Some(&owner.run_id))
        {
            return Err(error());
        }
        update(&mut session)?;
        writer(&session).await.map_err(|_| error())
    }

    async fn verify_subagent_owner(&self) -> Result<(), String> {
        let Some(owner) = &self.subagent_owner else {
            return Ok(());
        };
        super::super::subagent_registry::owns_execution(
            &self.session_id,
            &owner.run_id,
            &owner.execution_id,
        )
        .await
        .then_some(())
        .ok_or_else(error)
    }
}
