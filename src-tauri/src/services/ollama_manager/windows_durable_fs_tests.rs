use super::durable_fs::{OllamaDurableFs, OllamaFsErrorKind, PlatformOllamaDurableFs};

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

#[test]
fn verified_windows_delete_uses_an_existing_directory_as_the_volume_hint() {
    let implementation = include_str!("durable_fs_windows_verified.rs");
    let handles = include_str!("durable_fs_windows_verified/handles.rs");

    assert!(!implementation.contains("open_volume"));
    assert!(!handles.contains("fn open_volume"));
    assert!(!handles.contains(r#"\\.\\"#));
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

/// Caracterise ce que Windows renvoie reellement quand un dossier de bundle ne
/// peut pas etre renomme parce qu'un fichier qu'il contient est ouvert. Le
/// contrat de mise a jour ne retente que les violations de partage ; or la
/// mesure montre que ce scenario produit ERROR_ACCESS_DENIED pour tous les modes
/// de partage, y compris le plus permissif. La reprise bornee ne s'applique donc
/// pas a ce cas, et l'operation echoue fermee en conservant les deux versions.
/// Ce test fige ce constat : l'elargir a ERROR_ACCESS_DENIED serait un
/// amendement au contrat, pas une correction, et ferait echouer ce test.
#[test]
fn a_locked_bundle_directory_is_refused_as_a_permission_error() {
    use std::os::windows::fs::OpenOptionsExt;

    const ERROR_ACCESS_DENIED: u32 = 5;
    for partage in [0u32, 1, 3, 7] {
        let racine = tempfile::tempdir().expect("dossier temporaire");
        let source = racine.path().join("ollama-bundle");
        let destination = racine.path().join("ollama-bundle-backup");
        std::fs::create_dir_all(&source).expect("dossier source");
        let verrouille = source.join("ollama.exe");
        std::fs::write(&verrouille, b"binaire").expect("fichier verrouille");

        let fs = PlatformOllamaDurableFs::default();
        let tenu = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(partage)
            .open(&verrouille)
            .expect("handle sur le binaire");

        let refus = fs
            .rename_durable(&source, &destination)
            .expect_err("un dossier dont un fichier est ouvert ne peut pas etre renomme");
        assert_eq!(
            (refus.kind(), refus.os_code()),
            (OllamaFsErrorKind::PermissionDenied, Some(ERROR_ACCESS_DENIED)),
            "mode de partage {partage} : Windows refuse par un defaut de droits, non par une violation de partage",
        );

        drop(tenu);
        fs.rename_durable(&source, &destination)
            .expect("le renommage reussit une fois le fichier referme");
        assert!(destination.is_dir());
        assert!(!source.exists());
    }
}
