use super::node_compatible_path;
use std::path::PathBuf;

#[test]
fn removes_the_windows_verbatim_prefix_before_launching_node() {
    let path = PathBuf::from(r"\\?\C:\Program Files\Beaver\resources\extension-host");

    assert_eq!(
        node_compatible_path(path),
        PathBuf::from(r"C:\Program Files\Beaver\resources\extension-host")
    );
}
