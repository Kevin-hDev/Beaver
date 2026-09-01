use super::shell_helper::parse;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

const FLAG: &str = "--beaver-terminal-shell-helper";

#[test]
fn exact_helper_request_is_accepted_and_canonicalized() {
    let fixture = ExecutableFixture::new("terminal-helper-shell");
    let request = parse(arguments("42", "--", fixture.path(), &["-l"])).unwrap();

    assert_eq!(request.expected_parent, 42);
    assert_eq!(request.executable, fixture.canonical());
    assert_eq!(request.arguments, vec![OsString::from("-l")]);
}

#[test]
fn missing_or_surplus_helper_arguments_are_rejected() {
    let fixture = ExecutableFixture::new("terminal-helper-shell");
    assert!(parse(vec![OsString::from(FLAG)]).is_err());

    let too_many = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
    assert!(parse(arguments("42", "--", fixture.path(), &too_many)).is_err());
}

#[test]
fn zero_and_non_numeric_parent_ids_are_rejected() {
    let fixture = ExecutableFixture::new("terminal-helper-shell");
    assert!(parse(arguments("0", "--", fixture.path(), &[])).is_err());
    assert!(parse(arguments("parent", "--", fixture.path(), &[])).is_err());
}

#[test]
fn incorrect_flag_or_separator_is_rejected() {
    let fixture = ExecutableFixture::new("terminal-helper-shell");
    let mut wrong_flag = arguments("42", "--", fixture.path(), &[]);
    wrong_flag[0] = OsString::from("--beaver-terminal-helper");
    assert!(parse(wrong_flag).is_err());
    assert!(parse(arguments("42", "-", fixture.path(), &[])).is_err());
}

#[test]
fn unsafe_executable_paths_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let fixture = ExecutableFixture::new_in(root.path(), "terminal-helper-shell", true);
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).unwrap();

    assert!(parse(arguments("42", "--", Path::new("relative-shell"), &[])).is_err());
    assert!(parse(arguments("42", "--", &root.path().join("missing"), &[])).is_err());
    assert!(parse(arguments("42", "--", root.path(), &[])).is_err());
    assert!(parse(arguments(
        "42",
        "--",
        &nested.join("..").join(fixture.path().file_name().unwrap()),
        &[],
    ))
    .is_err());
}

#[test]
fn non_executable_file_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let fixture = ExecutableFixture::new_in(root.path(), "terminal-helper-data.txt", false);
    assert!(parse(arguments("42", "--", fixture.path(), &[])).is_err());
}

#[test]
fn request_larger_than_thirty_two_kib_is_rejected() {
    let fixture = ExecutableFixture::new("terminal-helper-shell");
    let oversized = "x".repeat(32 * 1024 + 1);
    assert!(parse(arguments("42", "--", fixture.path(), &[oversized.as_str()],)).is_err());
}

fn arguments(parent: &str, separator: &str, executable: &Path, tail: &[&str]) -> Vec<OsString> {
    let mut values = vec![
        OsString::from(FLAG),
        OsString::from(parent),
        OsString::from(separator),
        executable.as_os_str().to_owned(),
    ];
    values.extend(tail.iter().map(OsString::from));
    values
}

struct ExecutableFixture {
    _root: Option<tempfile::TempDir>,
    path: PathBuf,
}

impl ExecutableFixture {
    fn new(name: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join(executable_name(name));
        make_file(&path, true);
        Self {
            _root: Some(root),
            path,
        }
    }

    fn new_in(root: impl AsRef<Path>, name: &str, executable: bool) -> Self {
        let name = if executable {
            executable_name(name)
        } else {
            name.to_string()
        };
        let path = root.as_ref().join(name);
        make_file(&path, executable);
        Self { _root: None, path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn canonical(&self) -> PathBuf {
        dunce::canonicalize(&self.path).unwrap()
    }
}

fn make_file(path: &Path, executable: bool) {
    std::fs::write(path, b"fixture").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if executable { 0o700 } else { 0o600 };
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    #[cfg(windows)]
    let _ = executable;
}

fn executable_name(name: &str) -> String {
    #[cfg(windows)]
    return format!("{name}.exe");
    #[cfg(not(windows))]
    name.to_string()
}
