use landlock::RulesetCreatedAttr;
use landlock::{Access, AccessFs, ABI, CompatLevel, Compatible, Ruleset, RulesetAttr};
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

const PLATFORM_READ_DIRS: [&str; 8] = [
    "/bin", "/sbin", "/usr", "/lib", "/lib64", "/etc", "/dev", "/proc",
];
const PACKAGE_PREFIXES: [&str; 2] = ["/usr/local", "/home/linuxbrew/.linuxbrew"];
const SYSTEM_TEMP_DIR: &str = "/tmp";
const WRITABLE_DEVICES: [&str; 7] = [
    "/dev/null",
    "/dev/zero",
    "/dev/full",
    "/dev/random",
    "/dev/urandom",
    "/dev/tty",
    "/dev/ptmx",
];

pub(super) fn run(
    executable: &Path,
    arguments: &[OsString],
    writable_roots: &[PathBuf],
    temp_dir: &Path,
) -> Result<i32, String> {
    apply(writable_roots, temp_dir)?;
    let error = std::process::Command::new(executable).args(arguments).exec();
    Err(error.to_string())
}

fn apply(writable_roots: &[PathBuf], temp_dir: &Path) -> Result<(), String> {
    // ABI V3 protège aussi les troncatures tout en restant compatible avec
    // davantage de noyaux que les versions Landlock plus récentes.
    let abi = ABI::V3;
    let access_all = AccessFs::from_all(abi);
    let access_read = AccessFs::from_read(abi);
    let file_read_write = AccessFs::ReadFile | AccessFs::WriteFile | AccessFs::Truncate;
    let device_read_write = AccessFs::ReadFile | AccessFs::WriteFile | AccessFs::Truncate;
    let device_dir_read_write = device_read_write | AccessFs::ReadDir;
    let tools = super::tool_roots::collect(
        writable_roots,
        &PLATFORM_READ_DIRS,
        &PACKAGE_PREFIXES,
        None,
    );
    let write_dirs = writable_roots
        .iter()
        .map(PathBuf::as_path)
        .chain(std::iter::once(temp_dir))
        .chain(tools.write_dirs.iter().map(PathBuf::as_path))
        .chain(
            std::iter::once(Path::new(SYSTEM_TEMP_DIR)).filter(|path| path.is_dir()),
        );
    let writable_devices = WRITABLE_DEVICES
        .iter()
        .map(Path::new)
        .filter(|path| path.exists());
    let writable_device_dirs = [Path::new("/dev/pts")]
        .into_iter()
        .filter(|path| path.is_dir());
    let ruleset = Ruleset::default()
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(access_all)
        .map_err(|_| sandbox_error())?
        .create()
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(&tools.read_dirs, access_read))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(&tools.read_files, AccessFs::ReadFile))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(write_dirs, access_all))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(&tools.write_files, file_read_write))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(writable_devices, device_read_write))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(
            writable_device_dirs,
            device_dir_read_write,
        ))
        .map_err(|_| sandbox_error())?
        .no_new_privs(true);
    let status = ruleset.restrict_self().map_err(|_| sandbox_error())?;
    if status.ruleset == landlock::RulesetStatus::NotEnforced {
        return Err(sandbox_error());
    }
    Ok(())
}

fn sandbox_error() -> String {
    super::launch::sandbox_error()
}
