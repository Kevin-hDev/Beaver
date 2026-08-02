use std::ffi::{OsStr, OsString};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

// API Apple dépréciée : son absence doit continuer à bloquer le shell restreint.
const SANDBOX_EXEC: &str = "/usr/bin/sandbox-exec";
const PLATFORM_READ_DIRS: [&str; 10] = [
    "/System",
    "/bin",
    "/sbin",
    "/usr",
    "/Library/Apple",
    "/Library/Developer",
    "/Library/Preferences",
    "/private/etc",
    "/private/var/db",
    "/dev",
];
const PACKAGE_PREFIXES: [&str; 2] = ["/opt/homebrew", "/usr/local"];

struct SandboxRoots {
    write_dirs: Vec<PathBuf>,
    write_files: Vec<PathBuf>,
    read_dirs: Vec<PathBuf>,
    read_files: Vec<PathBuf>,
}

pub(super) fn run(
    executable: &Path,
    arguments: &[OsString],
    writable_roots: &[PathBuf],
    temp_dir: &Path,
) -> Result<i32, String> {
    if !Path::new(SANDBOX_EXEC).is_file() {
        return Err(super::launch::sandbox_error());
    }
    let roots = sandbox_roots(writable_roots, temp_dir);
    let mut command = std::process::Command::new(SANDBOX_EXEC);
    add_parameters(&mut command, "BEAVER_RW_DIR", &roots.write_dirs);
    add_parameters(&mut command, "BEAVER_RW_FILE", &roots.write_files);
    add_parameters(&mut command, "BEAVER_RO_DIR", &roots.read_dirs);
    add_parameters(&mut command, "BEAVER_RO_FILE", &roots.read_files);
    command.arg("-p").arg(policy(&roots)).arg(executable).args(arguments);
    let error = command.exec();
    Err(error.to_string())
}

fn sandbox_roots(writable_roots: &[PathBuf], temp_dir: &Path) -> SandboxRoots {
    let tools = super::tool_roots::collect(
        writable_roots,
        &PLATFORM_READ_DIRS,
        &PACKAGE_PREFIXES,
        None,
    );
    SandboxRoots {
        write_dirs: writable_roots
            .iter()
            .cloned()
            .chain(std::iter::once(temp_dir.to_path_buf()))
            .chain(tools.write_dirs)
            .collect(),
        write_files: tools.write_files,
        read_dirs: tools.read_dirs,
        read_files: tools.read_files,
    }
}

fn add_parameters(
    command: &mut std::process::Command,
    prefix: &str,
    paths: &[PathBuf],
) {
    for (index, path) in paths.iter().enumerate() {
        add_parameter(command, prefix, index, path.as_os_str());
    }
}

fn add_parameter(command: &mut std::process::Command, prefix: &str, index: usize, value: &OsStr) {
    command.arg("-D").arg(format!("{prefix}_{index}={}", value.to_string_lossy()));
}

fn policy(roots: &SandboxRoots) -> String {
    let mut policy = include_str!("macos_seatbelt_base.sbpl").to_string();
    policy.push_str(include_str!("macos_platform.sbpl"));
    policy.push_str(include_str!("macos_base.sbpl"));
    append_rules(&mut policy, "BEAVER_RW_DIR", roots.write_dirs.len(), true, false);
    append_rules(&mut policy, "BEAVER_RW_FILE", roots.write_files.len(), true, true);
    append_rules(&mut policy, "BEAVER_RO_DIR", roots.read_dirs.len(), false, false);
    append_rules(&mut policy, "BEAVER_RO_FILE", roots.read_files.len(), false, true);
    policy
}

fn append_rules(policy: &mut String, prefix: &str, count: usize, writable: bool, literal: bool) {
    let access = match (writable, literal) {
        (true, false) => "file-read* file-write* file-test-existence file-map-executable",
        (true, true) => "file-read* file-write* file-test-existence",
        (false, false) => "file-read* file-test-existence file-map-executable",
        (false, true) => "file-read* file-test-existence",
    };
    let matcher = if literal { "literal" } else { "subpath" };
    for index in 0..count {
        policy.push_str(&format!(
            "(allow {access} ({matcher} (param \"{prefix}_{index}\")))\n"
        ));
    }
}

#[cfg(test)]
#[path = "macos_tests.rs"]
mod tests;
