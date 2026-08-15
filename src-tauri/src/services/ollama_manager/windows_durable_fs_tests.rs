use super::durable_fs::{OllamaDurableFs, PlatformOllamaDurableFs};

#[test]
fn windows_native_primitive_flushes_parent_after_replace() {
    let root = tempfile::tempdir().expect("temporary directory");
    let fs = PlatformOllamaDurableFs::default();
    let tmp = root.path().join("journal.tmp");
    let final_path = root.path().join("journal.json");

    fs.write_new_atomic(&tmp, &final_path, b"first")
        .expect("new durable write");
    fs.replace_atomic(&tmp, &final_path, b"second")
        .expect("replacement durable write");
    assert_eq!(std::fs::read(final_path).unwrap(), b"second");
}

#[test]
fn verified_windows_delete_never_falls_back_to_path_recursive_removal() {
    let source = include_str!("durable_fs_windows.rs");
    let method = source
        .split_once("fn remove_tree_verified")
        .and_then(|(_, remainder)| remainder.split_once("\n    fn sync_file"))
        .map(|(body, _)| body)
        .expect("verified Windows deletion method");

    assert!(!method.contains("remove_tree(root.path())"));
    assert!(method.contains("verified::remove_tree(root)"));
}

#[cfg(windows)]
#[test]
fn verified_windows_delete_removes_nested_tree_using_the_native_handle() {
    use super::path_identity::{NativePathIdentityResolver, PathIdentityResolver};

    let root = tempfile::tempdir().expect("temporary directory");
    let tree = root.path().join("trash");
    std::fs::create_dir_all(tree.join("nested/deep")).expect("nested tree");
    std::fs::write(tree.join("nested/deep/file"), b"trash").expect("tree file");
    let canonical = NativePathIdentityResolver
        .canonical_directory(&tree)
        .expect("native identity");
    PlatformOllamaDurableFs::default()
        .remove_tree_verified(&canonical)
        .expect("handle-relative removal");

    assert!(!tree.exists());
}
