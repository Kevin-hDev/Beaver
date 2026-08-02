use std::ffi::{OsStr, OsString};
use std::os::unix::process::CommandExt;
use std::os::unix::ffi::OsStrExt;
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
const MAX_DARWIN_TEMP_BYTES: usize = 4_096;

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
    if let Some(rule) = xcrun_cache_rule() {
        policy.push_str(&rule);
    }
    append_rules(&mut policy, "BEAVER_RW_DIR", roots.write_dirs.len(), true, false);
    append_rules(&mut policy, "BEAVER_RW_FILE", roots.write_files.len(), true, true);
    append_rules(&mut policy, "BEAVER_RO_DIR", roots.read_dirs.len(), false, false);
    append_rules(&mut policy, "BEAVER_RO_FILE", roots.read_files.len(), false, true);
    policy
}

fn xcrun_cache_rule() -> Option<String> {
    let path = darwin_user_temp_dir()?;
    let text = path.to_str()?.trim_end_matches('/');
    if text.is_empty()
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b'.'))
    {
        return None;
    }
    let escaped = text.replace('.', "\\.");
    Some(format!(
        "(allow file-read* file-write*\n  (regex #\"^{escaped}/xcrun_db[^/]*$\"))\n"
    ))
}

fn darwin_user_temp_dir() -> Option<PathBuf> {
    // SAFETY: le premier appel demande uniquement la taille du buffer à libc.
    let length = unsafe { libc::confstr(libc::_CS_DARWIN_USER_TEMP_DIR, std::ptr::null_mut(), 0) };
    if length == 0 || length > MAX_DARWIN_TEMP_BYTES {
        return None;
    }
    let mut buffer = vec![0_u8; length];
    // SAFETY: le buffer alloué utilise exactement la taille bornée annoncée par libc.
    let written = unsafe {
        libc::confstr(
            libc::_CS_DARWIN_USER_TEMP_DIR,
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    if written == 0 || written > buffer.len() {
        return None;
    }
    let value = std::ffi::CStr::from_bytes_until_nul(&buffer).ok()?;
    let path = PathBuf::from(OsStr::from_bytes(value.to_bytes()));
    dunce::canonicalize(path).ok().filter(|path| path.is_dir())
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
