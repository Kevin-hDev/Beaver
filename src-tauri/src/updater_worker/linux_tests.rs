use std::ffi::OsString;
use std::path::Path;

use super::install_spec;

#[test]
fn uses_only_pkexec_and_apt_get_with_separate_arguments() {
    let asset = Path::new("/tmp/CL GO/update.deb");
    let spec = install_spec(asset);
    assert_eq!(spec.program, Path::new("/usr/bin/pkexec"));
    assert_eq!(
        spec.args,
        vec![
            OsString::from("/usr/bin/apt-get"),
            OsString::from("install"),
            OsString::from("-y"),
            asset.as_os_str().to_owned(),
        ]
    );
    let rendered = format!("{spec:?}");
    for forbidden in ["bash", "sudo", "terminal", "-lc"] {
        assert!(!rendered.contains(forbidden));
    }
}
