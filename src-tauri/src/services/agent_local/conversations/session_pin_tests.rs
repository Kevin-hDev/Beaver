use super::*;
use crate::services::agent_local::session_order;
use crate::services::agent_local::session_store;

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    session_order::test_lock().lock().await
}

async fn nouvelle_session() -> String {
    session_store::create_full("test", "llama3", "ollama", false, None)
        .await
        .expect("création de session")
        .id
}

#[tokio::test]
async fn epingler_pose_la_date_et_la_garde_au_second_appel() {
    let _g = guard().await;
    let id = nouvelle_session().await;

    pin(&id).await.unwrap();
    let premiere = session_store::get(&id)
        .await
        .unwrap()
        .pinned_at
        .expect("épinglée");
    pin(&id).await.unwrap();
    let seconde = session_store::get(&id)
        .await
        .unwrap()
        .pinned_at
        .expect("toujours épinglée");

    assert_eq!(premiere, seconde);
}

#[tokio::test]
async fn desepingler_efface_la_date() {
    let _g = guard().await;
    let id = nouvelle_session().await;
    pin(&id).await.unwrap();

    unpin(&id).await.unwrap();

    assert!(session_store::get(&id).await.unwrap().pinned_at.is_none());
}

#[tokio::test]
async fn desepingler_une_session_non_epinglee_ne_fait_rien() {
    let _g = guard().await;
    let id = nouvelle_session().await;

    assert!(unpin(&id).await.is_ok());
    assert!(session_store::get(&id).await.unwrap().pinned_at.is_none());
}

/* Épinglée, la session quitte sa liste d'origine : son rang là-bas doit
   disparaître, sinon elle garderait deux rangs. */
#[tokio::test]
async fn epingler_oublie_le_rang_de_la_liste_d_origine() {
    let _g = guard().await;
    let id = nouvelle_session().await;
    session_order::set(None, vec![id.clone()]).await.unwrap();

    pin(&id).await.unwrap();

    assert!(!session_order::ranks().await.contains_key(&id));
}

#[tokio::test]
async fn desepingler_oublie_le_rang_de_la_liste_epinglee() {
    let _g = guard().await;
    let id = nouvelle_session().await;
    pin(&id).await.unwrap();
    session_order::set_pinned(vec![id.clone()]).await.unwrap();

    unpin(&id).await.unwrap();

    assert!(!session_order::ranks().await.contains_key(&id));
}

#[tokio::test]
async fn un_identifiant_invalide_est_refuse() {
    assert!(pin("../ailleurs").await.is_err());
    assert!(unpin("../ailleurs").await.is_err());
}

/* L'index visible filtre les archivées avant tout : une épinglée archivée
   disparaît de la liste, et son épingle reste pour la restauration. */
#[tokio::test]
async fn une_epinglee_archivee_garde_son_epingle_hors_de_la_liste_active() {
    let _g = guard().await;
    let id = nouvelle_session().await;
    pin(&id).await.unwrap();

    crate::services::agent_local::session_archive::archive(&id)
        .await
        .unwrap();

    assert!(session_store::list()
        .await
        .unwrap()
        .iter()
        .all(|m| m.id != id));
    assert!(session_store::get(&id).await.unwrap().pinned_at.is_some());
}
