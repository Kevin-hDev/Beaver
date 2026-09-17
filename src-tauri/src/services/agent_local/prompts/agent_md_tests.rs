use super::*;
use tempfile::TempDir;

#[tokio::test]
async fn load_global_only() {
    let tmp = TempDir::new().unwrap();
    tokio::fs::write(tmp.path().join("AGENTS.md"), "# Global rules\nBe concise.")
        .await
        .unwrap();
    let result = load_agent_md_from(tmp.path(), None).await;
    let content = result.unwrap();
    assert!(content.contains("Global rules"));
    assert!(content.contains("global instructions"));
}

#[tokio::test]
async fn load_global_and_project() {
    let data = TempDir::new().unwrap();
    tokio::fs::write(data.path().join("AGENTS.md"), "Global stuff")
        .await
        .unwrap();

    let proj = TempDir::new().unwrap();
    tokio::fs::write(proj.path().join("AGENTS.md"), "Project stuff")
        .await
        .unwrap();

    let result = load_agent_md_from(data.path(), Some(proj.path())).await;
    let content = result.unwrap();
    assert!(content.contains("Global stuff"));
    assert!(content.contains("Project stuff"));
    let global_pos = content.find("Global stuff").unwrap();
    let project_pos = content.find("Project stuff").unwrap();
    assert!(project_pos > global_pos);
}

#[tokio::test]
async fn no_files_returns_none() {
    let tmp = TempDir::new().unwrap();
    let result = load_agent_md_from(tmp.path(), Some(tmp.path())).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn project_only_no_global() {
    let data = TempDir::new().unwrap();
    let proj = TempDir::new().unwrap();
    tokio::fs::write(proj.path().join("AGENTS.md"), "Project only")
        .await
        .unwrap();
    let result = load_agent_md_from(data.path(), Some(proj.path())).await;
    let content = result.unwrap();
    assert!(content.contains("Project only"));
    assert!(!content.contains("imported global instructions"));
}

#[tokio::test]
async fn no_parent_agent_md_loaded() {
    let root = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    let child = root.path().join("child");
    tokio::fs::create_dir_all(&child).await.unwrap();

    tokio::fs::write(root.path().join("AGENTS.md"), "Parent rules")
        .await
        .unwrap();
    tokio::fs::write(child.join("AGENTS.md"), "Child rules")
        .await
        .unwrap();

    let result = load_agent_md_from(data.path(), Some(&child)).await;
    let content = result.unwrap();
    assert!(content.contains("Child rules"));
    assert!(
        !content.contains("Parent rules"),
        "Les AGENTS.md parents ne doivent PAS être chargés"
    );
}

#[tokio::test]
async fn cl_go_dir_loaded() {
    let proj = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    let cl_go = proj.path().join(".cl-go");
    tokio::fs::create_dir_all(&cl_go).await.unwrap();
    tokio::fs::write(cl_go.join("AGENTS.md"), "ClGo specific rules")
        .await
        .unwrap();

    let result = load_agent_md_from(data.path(), Some(proj.path())).await;
    let content = result.unwrap();
    assert!(content.contains("ClGo specific rules"));
}

#[tokio::test]
async fn rules_dir_sorted_alphabetically() {
    let proj = TempDir::new().unwrap();
    let data = TempDir::new().unwrap();
    let rules = proj.path().join(".cl-go").join("rules");
    tokio::fs::create_dir_all(&rules).await.unwrap();

    tokio::fs::write(rules.join("c-rule.md"), "Rule C")
        .await
        .unwrap();
    tokio::fs::write(rules.join("a-rule.md"), "Rule A")
        .await
        .unwrap();
    tokio::fs::write(rules.join("b-rule.md"), "Rule B")
        .await
        .unwrap();

    let result = load_agent_md_from(data.path(), Some(proj.path())).await;
    let content = result.unwrap();

    let pos_a = content.find("Rule A").unwrap();
    let pos_b = content.find("Rule B").unwrap();
    let pos_c = content.find("Rule C").unwrap();
    assert!(pos_a < pos_b, "Rule A doit précéder Rule B");
    assert!(pos_b < pos_c, "Rule B doit précéder Rule C");
}
