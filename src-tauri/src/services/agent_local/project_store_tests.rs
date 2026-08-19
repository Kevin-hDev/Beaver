use super::{
    canonical_existing_dir, path_is_inside_project, project_matches_canonical, validate_projects,
    Project, MAX_PROJECTS, PROJECT_STORE_UNAVAILABLE,
};
use chrono::Utc;

fn project(id: &str) -> Project {
    Project {
        id: id.to_string(),
        name: "project".to_string(),
        path: "/bounded/project".to_string(),
        order: 0,
        created_at: Utc::now(),
    }
}

#[test]
fn canonical_existing_dir_normalizes_dot_segments() {
    let tmp = tempfile::tempdir().expect("temp");
    let nested = tmp.path().join("nested");
    std::fs::create_dir_all(&nested).expect("nested");

    let canonical = canonical_existing_dir(&nested.join(".")).expect("canonical");

    assert_eq!(canonical, dunce::canonicalize(&nested).expect("expected"));
}

#[test]
fn project_match_accepts_equivalent_path() {
    let tmp = tempfile::tempdir().expect("temp");
    let canonical = dunce::canonicalize(tmp.path()).expect("canonical");
    let equivalent = tmp.path().join(".");

    assert!(project_matches_canonical(
        &equivalent.to_string_lossy(),
        &canonical
    ));
}

#[test]
fn inside_project_allows_child_and_rejects_sibling() {
    let tmp = tempfile::tempdir().expect("temp");
    let project = tmp.path().join("project");
    let child = project.join("child");
    let sibling = tmp.path().join("sibling");
    std::fs::create_dir_all(&child).expect("child");
    std::fs::create_dir_all(&sibling).expect("sibling");

    let child = dunce::canonicalize(child).expect("canonical child");
    let sibling = dunce::canonicalize(sibling).expect("canonical sibling");

    assert!(path_is_inside_project(&child, &project.to_string_lossy()));
    assert!(!path_is_inside_project(
        &sibling,
        &project.to_string_lossy()
    ));
}

#[test]
fn persisted_projects_reject_duplicate_identifiers() {
    let duplicate = vec![project("same"), project("same")];

    assert_eq!(
        validate_projects(&duplicate),
        Err(PROJECT_STORE_UNAVAILABLE.to_string())
    );
}

#[test]
fn persisted_project_collection_is_bounded() {
    let projects = (0..=MAX_PROJECTS)
        .map(|index| project(&format!("project-{index}")))
        .collect::<Vec<_>>();

    assert_eq!(
        validate_projects(&projects),
        Err(PROJECT_STORE_UNAVAILABLE.to_string())
    );
}

#[tokio::test]
async fn corrupt_project_document_is_backed_up_then_reset() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("projects.json");
    let corrupt = b"{not-json";
    std::fs::write(&path, corrupt).unwrap();

    let projects = super::read_all_from(&path).await.unwrap();

    assert!(projects.is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), b"[]");
    assert_eq!(
        std::fs::read(root.path().join("projects.json.corrupted")).unwrap(),
        corrupt
    );
}

#[tokio::test]
async fn invalid_project_collection_is_backed_up_then_reset() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("projects.json");
    let duplicate = vec![project("same"), project("same")];
    let corrupt = serde_json::to_vec(&duplicate).unwrap();
    std::fs::write(&path, &corrupt).unwrap();

    let projects = super::read_all_from(&path).await.unwrap();

    assert!(projects.is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), b"[]");
    assert_eq!(
        std::fs::read(root.path().join("projects.json.corrupted")).unwrap(),
        corrupt
    );
}
