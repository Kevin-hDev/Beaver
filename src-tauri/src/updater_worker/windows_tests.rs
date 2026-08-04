use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::{install_spec, launch_spec, Installation};

#[test]
fn launches_nsis_directly_with_silent_and_destination_arguments() {
    let asset = Path::new(r"C:\Temp\Beaver setup.exe");
    let installation = Path::new(r"C:\Program Files\Beaver");
    let spec = install_spec(asset, installation);
    assert_eq!(spec.program, asset);
    assert_eq!(
        spec.args,
        vec![
            OsString::from("/S"),
            OsString::from(r"/D=C:\Program Files\Beaver"),
        ]
    );
    assert_eq!(spec.args.len(), 2);
}

#[test]
fn launches_beaver_with_exact_health_arguments() {
    let installation = Installation {
        working_directory: PathBuf::from(r"C:\Program Files\Beaver"),
        executable: PathBuf::from(r"C:\Program Files\Beaver\cl-go-dash.exe"),
        bundle: None,
    };
    let token = "ab".repeat(32);
    let spec = launch_spec(&installation, &token);
    assert_eq!(spec.program, installation.executable);
    assert_eq!(
        spec.args,
        vec![
            OsString::from("--clgo-update-health"),
            OsString::from(token),
        ]
    );
}
