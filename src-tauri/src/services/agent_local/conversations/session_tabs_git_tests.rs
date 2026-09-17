use super::*;

#[tokio::test]
async fn failed_session_save_leaves_tabs_unchanged() {
    let branch = "clone-55555555";
    let mut root = session_store::create_full("Root", "llama3", "ollama", false, None)
        .await
        .expect("create root");
    let root_id = root.id.clone();
    root.git_branch = Some(branch.to_string());
    session_store::save(&root).await.expect("save root");
    let mut clone = session_store::create_full("Clone", "llama3", "ollama", false, None)
        .await
        .expect("create clone");
    let clone_id = clone.id.clone();
    clone.clone_parent_session_id = Some(root_id.clone());
    clone.clone_parent_message_id = Some("m1".to_string());
    clone.clone_mode = Some(super::super::types_session::CloneMode::Cut);
    clone.clone_root_session_id = Some(root_id.clone());
    clone.git_branch = Some(branch.to_string());
    session_store::save(&clone).await.expect("save clone");
    let mut tabs = normalize_tabs(&root_id, None);
    tabs.tabs.push(SessionTab {
        tab_id: "branch-1".to_string(),
        session_id: clone_id.clone(),
        label: "Branche 1".to_string(),
        is_main: false,
        clone_parent_session_id: Some(root_id.clone()),
        clone_parent_message_id: Some("m1".to_string()),
        clone_mode: Some(super::super::types_session::CloneMode::Cut),
        git_branch: Some(branch.to_string()),
    });
    tabs.active_tab_id = "branch-1".to_string();
    let before = tabs.clone();

    let result = sync_git_branches_with_writer(&root_id, &mut tabs, |_| async {
        Err("injected session save failure".to_string())
    })
    .await;

    assert_eq!(result, Err("Action impossible".to_string()));
    assert_eq!(tabs, before);
    assert_eq!(
        session_store::get(&clone_id)
            .await
            .expect("reload clone")
            .git_branch
            .as_deref(),
        Some(branch)
    );
    session_store::delete_one(&clone_id).await.expect("cleanup clone");
    session_store::delete_one(&root_id).await.expect("cleanup root");
}
