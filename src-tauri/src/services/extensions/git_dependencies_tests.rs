use super::npm_runner::{resolve_cli, NpmRunner};
use super::source_validation::GitSource;
use git2::{Repository, Signature};
use std::path::Path;

fn dependency_archive(root: &Path) {
    let archive = std::fs::File::create(root.join("fixture-dependency.tgz")).unwrap();
    let encoder = flate2::write::GzEncoder::new(archive, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    let package = serde_json::json!({
        "name": "fixture-dependency",
        "version": "1.0.0",
        "scripts": { "postinstall": "node unsafe-dependency.js" }
    })
    .to_string();
    let script = b"require('fs').writeFileSync('../unsafe-dependency-ran','1')";
    for (path, bytes) in [
        ("package/package.json", package.as_bytes()),
        ("package/unsafe-dependency.js", script.as_slice()),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o600);
        header.set_cksum();
        builder.append_data(&mut header, path, bytes).unwrap();
    }
    builder.finish().unwrap();
    builder.into_inner().unwrap().finish().unwrap();
}

fn write_fixture(root: &Path) {
    dependency_archive(root);
    std::fs::write(root.join("index.js"), "export default () => {}").unwrap();
    std::fs::write(
        root.join("beaver-extension.json"),
        serde_json::json!({
            "id": "test.git.dependencies",
            "name": "Git dependencies",
            "version": "1.0.0",
            "beaverApi": "1",
            "runtime": "node",
            "main": "index.js",
            "access": "full"
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        root.join("package.json"),
        serde_json::json!({
            "name": "git-dependency-fixture",
            "version": "1.0.0",
            "scripts": { "postinstall": "node unsafe.js" },
            "dependencies": { "fixture-dependency": "file:./fixture-dependency.tgz" }
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        root.join("unsafe.js"),
        "require('fs').writeFileSync('unsafe-ran','1')",
    )
    .unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "registry=http://127.0.0.1:9/\nstrict-ssl=false\nignore-scripts=false\n",
    )
    .unwrap();
}

fn commit_all(root: &Path) {
    let repository = Repository::init(root).unwrap();
    let mut index = repository.index().unwrap();
    index
        .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repository.find_tree(tree_id).unwrap();
    let signature = Signature::now("Beaver test", "test@beaver.local").unwrap();
    repository
        .commit(Some("HEAD"), &signature, &signature, "fixture", &tree, &[])
        .unwrap();
}

#[test]
fn git_materialization_installs_dependencies_without_running_scripts_or_npmrc() {
    let temporary = tempfile::tempdir().unwrap();
    let repository = temporary.path().join("repository");
    let destination = temporary.path().join("destination");
    std::fs::create_dir(&repository).unwrap();
    std::fs::create_dir(&destination).unwrap();
    write_fixture(&repository);
    commit_all(&repository);
    let node = which::which("node").unwrap().canonicalize().unwrap();
    let cli = resolve_cli(temporary.path(), &node).unwrap();
    let npm = NpmRunner::for_test(node, cli);
    let source = GitSource {
        locator: "https://example.invalid/dependencies.git".to_string(),
        clone_url: url::Url::from_file_path(&repository).unwrap().to_string(),
        reference: None,
    };

    let installed = super::git_source::materialize(
        &source,
        &destination,
        &npm,
        &super::work_supervision::open_cancellation_for_test(),
    )
    .unwrap();

    assert!(installed
        .root
        .join("node_modules/fixture-dependency/package.json")
        .is_file());
    assert!(!installed.root.join("unsafe-ran").exists());
    assert!(!installed.root.join("unsafe-dependency-ran").exists());
    assert!(!installed.root.join(".npm-cache").exists());
    assert!(std::fs::read_to_string(installed.root.join(".npmrc"))
        .unwrap()
        .contains("http://127.0.0.1:9/"));
}
