use std::ffi::{OsStr, OsString};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

const SANDBOX_EXEC: &str = "/usr/bin/sandbox-exec";
const MAX_READ_ROOTS: usize = 64;

pub(super) fn run(
    executable: &Path,
    arguments: &[OsString],
    writable_roots: &[PathBuf],
    temp_dir: &Path,
) -> Result<i32, String> {
    if !Path::new(SANDBOX_EXEC).is_file() {
        return Err(super::launch::sandbox_error());
    }
    let read_roots = read_roots(writable_roots);
    let mut command = std::process::Command::new(SANDBOX_EXEC);
    for (index, root) in writable_roots
        .iter()
        .map(PathBuf::as_path)
        .chain(std::iter::once(temp_dir))
        .enumerate()
    {
        add_parameter(&mut command, "BEAVER_RW", index, root.as_os_str());
    }
    for (index, root) in read_roots.iter().enumerate() {
        add_parameter(&mut command, "BEAVER_RO", index, root.as_os_str());
    }
    let profile = policy(writable_roots.len() + 1, read_roots.len());
    command.arg("-p").arg(profile).arg(executable).args(arguments);
    let error = command.exec();
    Err(error.to_string())
}

fn add_parameter(command: &mut std::process::Command, prefix: &str, index: usize, value: &OsStr) {
    command.arg("-D").arg(format!("{prefix}_{index}={}", value.to_string_lossy()));
}

fn policy(write_count: usize, read_count: usize) -> String {
    let mut policy = include_str!("macos_seatbelt_base.sbpl").to_string();
    policy.push_str(include_str!("macos_platform.sbpl"));
    policy.push_str(include_str!("macos_base.sbpl"));
    for index in 0..write_count {
        policy.push_str(&format!(
            "(allow file-read* file-write* file-test-existence file-map-executable (subpath (param \"BEAVER_RW_{index}\")))\n"
        ));
    }
    for index in 0..read_count {
        policy.push_str(&format!(
            "(allow file-read* file-test-existence file-map-executable (subpath (param \"BEAVER_RO_{index}\")))\n"
        ));
    }
    policy
}

fn read_roots(writable_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = [
        "/System",
        "/bin",
        "/sbin",
        "/usr",
        "/Library/Apple",
        "/Library/Preferences",
        "/private/etc",
        "/private/var/db",
        "/dev",
    ]
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    for path in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        if roots.len() >= MAX_READ_ROOTS {
            break;
        }
        let Some(path) = path
            .is_absolute()
            .then(|| dunce::canonicalize(path).ok())
            .flatten()
            .filter(|path| path.is_dir())
        else {
            continue;
        };
        let overlaps_writable = writable_roots
            .iter()
            .any(|root| path.starts_with(root) || root.starts_with(&path));
        if !overlaps_writable && !roots.contains(&path) {
            roots.push(path);
        }
    }
    roots
}

#[cfg(test)]
#[path = "macos_tests.rs"]
mod tests;
