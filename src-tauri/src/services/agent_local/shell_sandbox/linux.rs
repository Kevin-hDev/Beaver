use landlock::RulesetCreatedAttr;
use landlock::{
    Access, AccessFs, ABI, CompatLevel, Compatible, Ruleset, RulesetAttr, RulesetStatus,
};
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

const PLATFORM_READ_DIRS: [&str; 7] = [
    "/bin", "/sbin", "/usr", "/lib", "/lib64", "/etc", "/dev",
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
    scope: &super::scope::Scope,
    temp_dir: &Path,
) -> Result<i32, String> {
    match super::linux_namespace::enter()? {
        super::linux_namespace::Entered::Parent(code) => Ok(code),
        super::linux_namespace::Entered::Child => execute(executable, arguments, scope, temp_dir, true),
        super::linux_namespace::Entered::Unavailable => {
            execute(executable, arguments, scope, temp_dir, false)
        }
    }
}

fn execute(
    executable: &Path,
    arguments: &[OsString],
    scope: &super::scope::Scope,
    temp_dir: &Path,
    private_proc: bool,
) -> Result<i32, String> {
    apply(scope, temp_dir, private_proc)?;
    let error = std::process::Command::new(executable).args(arguments).exec();
    Err(error.to_string())
}

fn apply(scope: &super::scope::Scope, temp_dir: &Path, private_proc: bool) -> Result<(), String> {
    // ABI V3 protège aussi les troncatures tout en restant compatible avec
    // davantage de noyaux que les versions Landlock plus récentes.
    let abi = ABI::V3;
    let access_all = AccessFs::from_all(abi);
    let access_read = AccessFs::from_read(abi);
    let file_read_write = AccessFs::ReadFile | AccessFs::WriteFile | AccessFs::Truncate;
    let device_read_write = AccessFs::ReadFile | AccessFs::WriteFile | AccessFs::Truncate;
    let device_dir_read_write = device_read_write | AccessFs::ReadDir;
    let tools = if scope.mode == super::scope::Mode::ProfileCapture {
        super::tool_roots::collect_read_only(
            &scope.roots,
            &PLATFORM_READ_DIRS,
            &PACKAGE_PREFIXES,
            None,
        )
    } else {
        super::tool_roots::collect(
            &scope.roots,
            &PLATFORM_READ_DIRS,
            &PACKAGE_PREFIXES,
            None,
        )
    };
    let workspace_mode = scope.mode == super::scope::Mode::Workspace;
    let write_dirs = scope
        .roots
        .iter()
        .map(PathBuf::as_path)
        .filter(|_| workspace_mode)
        .chain(std::iter::once(temp_dir))
        .chain(tools.write_dirs.iter().map(PathBuf::as_path).filter(|_| workspace_mode))
        .chain(
            std::iter::once(Path::new(SYSTEM_TEMP_DIR))
                .filter(|path| workspace_mode && path.is_dir()),
        );
    let read_dirs = scope
        .roots
        .iter()
        .map(PathBuf::as_path)
        .filter(|_| !workspace_mode)
        .chain(scope.read_dirs.iter().map(PathBuf::as_path))
        .chain(tools.read_dirs.iter().map(PathBuf::as_path))
        .chain(std::iter::once(Path::new("/proc")).filter(|_| private_proc));
    let read_files = scope
        .read_files
        .iter()
        .map(PathBuf::as_path)
        .chain(tools.read_files.iter().map(PathBuf::as_path));
    let writable_devices = WRITABLE_DEVICES
        .iter()
        .map(Path::new)
        .filter(|path| path.exists());
    let writable_device_dirs = [Path::new("/dev/pts")]
        .into_iter()
        .filter(|path| path.is_dir());
    // Landlock étend ReadDir à toute la hiérarchie. Les racines prévues pour une
    // liste de premier niveau restent donc sans règle plutôt que d'exposer les
    // noms contenus dans les dossiers privés exclus.
    let ruleset = Ruleset::default().set_compatibility(CompatLevel::HardRequirement);
    let ruleset = match ruleset.handle_access(access_all) {
        Ok(ruleset) => {
            record_diagnostics(temp_dir, &tools, false);
            ruleset
        }
        Err(_) => {
            record_diagnostics(temp_dir, &tools, true);
            return Err(sandbox_error());
        }
    };
    let ruleset = ruleset
        .create()
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(read_dirs, access_read))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(read_files, AccessFs::ReadFile))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(write_dirs, access_all))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(
            tools.write_files.iter().filter(|_| workspace_mode),
            file_read_write,
        ))
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
    if isolation_is_unavailable(&status.ruleset) {
        record_diagnostics(temp_dir, &tools, true);
        return Err(sandbox_error());
    }
    Ok(())
}

fn record_diagnostics(
    temp_dir: &Path,
    tools: &super::tool_roots::ToolRoots,
    isolation_unavailable: bool,
) {
    super::super::shell_sandbox_diagnostics::record(
        temp_dir,
        tools.path_limit_reached,
        tools.read_limit_reached,
        tools.write_limit_reached,
        tools.cache_unavailable,
        isolation_unavailable,
    );
}

fn isolation_is_unavailable(status: &RulesetStatus) -> bool {
    !matches!(status, RulesetStatus::FullyEnforced)
}

fn sandbox_error() -> String {
    super::launch::sandbox_error()
}

#[cfg(test)]
#[path = "linux_tests.rs"]
mod tests;
