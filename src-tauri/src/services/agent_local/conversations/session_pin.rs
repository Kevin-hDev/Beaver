/* Épingler une conversation en tête de la barre latérale.

   Calqué sur session_archive : l'état vit dans la session, sous son verrou,
   et l'ordre manuel vit ailleurs (session_order). Après chaque bascule la
   conversation oublie son rang : elle change de liste, et une conversation
   sans rang passe en tête de sa liste d'arrivée, par activité — c'est
   exactement là qu'on veut la voir. */

use chrono::Utc;

use super::session_order;
use super::session_store::{get, lock_session, save, validate_session_id};

pub async fn pin(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let lock = lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    if session.pinned_at.is_none() {
        session.pinned_at = Some(Utc::now());
        save(&session).await?;
    }
    session_order::clear_rank(id).await
}

pub async fn unpin(id: &str) -> Result<(), String> {
    validate_session_id(id)?;
    let lock = lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    if session.pinned_at.is_some() {
        session.pinned_at = None;
        save(&session).await?;
    }
    session_order::clear_rank(id).await
}

#[path = "session_pin_tests.rs"]
#[cfg(test)]
mod tests;
