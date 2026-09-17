use super::append_configured_outputs_root;
use std::path::PathBuf;

#[test]
fn configured_outputs_are_an_allowed_root() {
    let output_root = PathBuf::from("/configured/session-outputs");
    let mut roots = vec![PathBuf::from("/workspace")];

    append_configured_outputs_root(&mut roots, Some(output_root.clone()));

    assert!(roots.contains(&output_root));
}
