use std::ffi::OsString;
use std::path::Path;

use super::{launch_spec, restart_spec};

#[test]
fn launches_new_bundle_with_only_the_health_argument() {
    let bundle = Path::new("/Applications/Beaver.app");
    let token = "ab".repeat(32);
    let spec = launch_spec(bundle, &token);
    assert_eq!(spec.program, Path::new("/usr/bin/open"));
    assert_eq!(
        spec.args,
        vec![
            OsString::from("-n"),
            bundle.as_os_str().to_owned(),
            OsString::from("--args"),
            OsString::from("--clgo-update-health"),
            OsString::from(token),
        ]
    );
}

#[test]
fn restarts_previous_bundle_without_forwarding_update_arguments() {
    let bundle = Path::new("/Applications/CL-GO.app");
    let spec = restart_spec(bundle);
    assert_eq!(
        spec.args,
        vec![OsString::from("-n"), bundle.as_os_str().to_owned()]
    );
}
