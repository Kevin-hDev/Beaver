use crate::services::agent_local::tool_executor_helpers::{post_record_read, post_record_write};
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

/// Simule un read_file réussi et vérifie que le fichier est enregistré.
#[test]
fn post_record_read_file_registers_path() {
    let mut guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "hello").unwrap();
    let path = tmp.path();

    let args = json!({ "path": path.to_str().unwrap() });
    let tr = ToolResult::ok("hello".to_string());
    post_record_read(
        "read_file",
        &args,
        std::path::Path::new("."),
        &tr,
        &mut guard,
    );

    // Le fichier est maintenant enregistré → write autorisé
    assert!(guard.check_write(path).is_ok());
}

/// grep doit enregistrer tous les fichiers matchés comme "vus".
#[test]
fn post_record_read_grep_registers_matched_files() {
    let mut guard = WriteGuard::new();
    // Crée 2 fichiers réels
    let dir = tempfile::tempdir().unwrap();
    let f1 = dir.path().join("a.rs");
    let f2 = dir.path().join("b.rs");
    std::fs::write(&f1, "TODO fix\n").unwrap();
    std::fs::write(&f2, "TODO also\n").unwrap();

    // Simule un résultat de grep
    let content = format!("{}:1:TODO fix\n{}:1:TODO also", f1.display(), f2.display());
    let args = json!({ "pattern": "TODO", "path": dir.path().to_str().unwrap() });
    let tr = ToolResult::ok(content);
    post_record_read("grep", &args, dir.path(), &tr, &mut guard);

    // Les 2 fichiers sont maintenant "vus" → write autorisé sans read_file
    assert!(
        guard.check_write(&f1).is_ok(),
        "f1 devrait être autorisé après grep"
    );
    assert!(
        guard.check_write(&f2).is_ok(),
        "f2 devrait être autorisé après grep"
    );
}

/// glob doit enregistrer tous les fichiers trouvés.
#[test]
fn post_record_read_glob_registers_found_files() {
    let mut guard = WriteGuard::new();
    let dir = tempfile::tempdir().unwrap();
    let f1 = dir.path().join("x.txt");
    let f2 = dir.path().join("y.txt");
    std::fs::write(&f1, "").unwrap();
    std::fs::write(&f2, "").unwrap();

    let content = format!("{}\n{}", f1.display(), f2.display());
    let args = json!({ "pattern": "*.txt", "path": dir.path().to_str().unwrap() });
    let tr = ToolResult::ok(content);
    post_record_read("glob", &args, dir.path(), &tr, &mut guard);

    assert!(
        guard.check_write(&f1).is_ok(),
        "f1 devrait être autorisé après glob"
    );
    assert!(
        guard.check_write(&f2).is_ok(),
        "f2 devrait être autorisé après glob"
    );
}

/// list_dir doit enregistrer les fichiers de l'arborescence.
#[test]
fn post_record_read_list_dir_registers_files() {
    let mut guard = WriteGuard::new();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "").unwrap();
    std::fs::create_dir_all(dir.path().join("utils")).unwrap();
    std::fs::write(dir.path().join("utils").join("helper.rs"), "").unwrap();

    let content = "main.rs\nutils/\n  helper.rs";
    let args = json!({ "path": dir.path().to_str().unwrap() });
    let tr = ToolResult::ok(content.to_string());
    post_record_read("list_dir", &args, dir.path(), &tr, &mut guard);

    let main = dir.path().join("main.rs");
    let helper = dir.path().join("utils").join("helper.rs");
    assert!(
        guard.check_write(&main).is_ok(),
        "main.rs devrait être autorisé après list_dir"
    );
    assert!(
        guard.check_write(&helper).is_ok(),
        "helper.rs devrait être autorisé"
    );
}

/// Un write réussi doit enregistrer le fichier comme "vu" pour le tour suivant.
#[test]
fn post_record_write_registers_written_file() {
    let mut guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "avant").unwrap();
    let path = tmp.path();

    // Avant write : bloqué (non lu)
    assert!(guard.check_write(path).is_err());

    // Simule un write_file réussi
    let args = json!({ "path": path.to_str().unwrap() });
    let tr = ToolResult::ok("ok".to_string());
    post_record_write(
        "write_file",
        &args,
        std::path::Path::new("."),
        &tr,
        &mut guard,
    );

    // Après write : autorisé (le fichier est "vu")
    assert!(
        guard.check_write(path).is_ok(),
        "après un write, le fichier devrait être autorisé au tour suivant"
    );
}

/// Un write en erreur ne doit PAS enregistrer le fichier.
#[test]
fn post_record_write_error_does_not_register() {
    let mut guard = WriteGuard::new();
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "avant").unwrap();
    let path = tmp.path();

    let args = json!({ "path": path.to_str().unwrap() });
    let tr = ToolResult::err("échec écriture");
    post_record_write(
        "write_file",
        &args,
        std::path::Path::new("."),
        &tr,
        &mut guard,
    );

    // Toujours bloqué car le write a échoué
    assert!(guard.check_write(path).is_err());
}

/// process_image doit enregistrer output_path après write.
#[test]
fn post_record_write_process_image_registers_output() {
    let mut guard = WriteGuard::new();
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.png");
    std::fs::write(&out, "").unwrap(); // existe

    let args = json!({ "output_path": out.to_str().unwrap() });
    let tr = ToolResult::ok("done".to_string());
    post_record_write(
        "process_image",
        &args,
        std::path::Path::new("."),
        &tr,
        &mut guard,
    );

    assert!(guard.check_write(&out).is_ok());
}
