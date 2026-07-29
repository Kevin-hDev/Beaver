use super::*;
use super::test_support::*;
use crate::services::git::{branch, branch_delete};
use uuid::Uuid;

#[tokio::test]
async fn cleanup_unlinks_branch_from_other_sessions() {
    let root_id = Uuid::new_v4().to_string();
    let clone_a_id = Uuid::new_v4().to_string();
    let clone_b_id = Uuid::new_v4().to_string();
    let branch_name = "clone-11111111";
    let repo = init_repo_with_commit();
    save_clone_tabs(&root_id, &clone_a_id, Some(&clone_b_id), branch_name).await;
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tab_id_for_session(&tabs, &clone_a_id);
    branch::create_branch(repo.path(), branch_name).expect("seed shared branch");

    let tabs = close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("master"))
        .await
        .expect("cleanup shared branch");

    assert!(!branch_delete::branch_exists(repo.path(), branch_name).expect("branch check"));
    let archived = super::session_store::get(&clone_a_id)
        .await
        .expect("archived clone");
    assert!(archived.archived_at.is_some());
    let linked = super::session_store::get(&clone_b_id)
        .await
        .expect("other clone");
    assert_eq!(linked.git_branch, None);
    assert!(tabs.tabs.iter().all(|tab| tab.session_id != clone_a_id));
    assert!(tabs
        .tabs
        .iter()
        .find(|tab| tab.session_id == clone_b_id)
        .is_some_and(|tab| tab.git_branch.is_none()));

    cleanup_sessions(&root_id, &[clone_a_id, clone_b_id]).await;
}

#[tokio::test]
async fn cleanup_prefers_main_over_temporary_checkpoint() {
    let root_id = Uuid::new_v4().to_string();
    let clone_id = Uuid::new_v4().to_string();
    let branch_name = "clone-33333333";
    let repo = init_repo_with_commit();
    branch::create_branch(repo.path(), "Delete-branch").expect("create fallback");
    branch::create_branch(repo.path(), "main").expect("create main");
    branch::create_branch(repo.path(), branch_name).expect("create clone branch");
    save_clone_tabs(&root_id, &clone_id, None, branch_name).await;
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tab_id_for_session(&tabs, &clone_id);

    close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("Delete-branch"))
        .await
        .expect("cleanup with stale checkpoint");

    assert!(!branch_delete::branch_exists(repo.path(), branch_name).expect("branch check"));
    assert_eq!(branch::get_context(repo.path()).branch, "main");

    cleanup_sessions(&root_id, &[clone_id]).await;
}

#[tokio::test]
async fn cleanup_deletes_manually_linked_branch() {
    let root_id = Uuid::new_v4().to_string();
    let clone_id = Uuid::new_v4().to_string();
    let branch_name = "feature/shared";
    let repo = init_repo_with_commit();
    branch::create_branch(repo.path(), branch_name).expect("create manual branch");
    save_clone_tabs(&root_id, &clone_id, None, branch_name).await;
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tab_id_for_session(&tabs, &clone_id);

    close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("master"))
        .await
        .expect("cleanup manual branch");

    assert!(!branch_delete::branch_exists(repo.path(), branch_name).expect("branch check"));
    let archived = super::session_store::get(&clone_id)
        .await
        .expect("archived clone");
    assert!(archived.archived_at.is_some());
    assert_eq!(archived.git_branch, None);

    cleanup_sessions(&root_id, &[clone_id]).await;
}

#[tokio::test]
async fn cleanup_refuses_to_delete_main_branch() {
    let root_id = Uuid::new_v4().to_string();
    let clone_id = Uuid::new_v4().to_string();
    let branch_name = "main";
    let repo = init_repo_with_commit();
    branch::create_branch(repo.path(), branch_name).expect("create main");
    save_clone_tabs(&root_id, &clone_id, None, branch_name).await;
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tab_id_for_session(&tabs, &clone_id);

    let result =
        close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("master")).await;

    assert_eq!(result, Err(GitActionError::ProtectedBranch));
    assert!(branch_delete::branch_exists(repo.path(), branch_name).expect("branch check"));

    cleanup_sessions(&root_id, &[clone_id]).await;
}
