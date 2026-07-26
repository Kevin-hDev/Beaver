use std::ffi::OsString;
use std::path::Path;

use super::install_spec;

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
