use std::ffi::{OsStr, OsString};
use std::os::unix::process::CommandExt;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

// API Apple dépréciée : son absence doit continuer à bloquer la limite de dossiers du shell.
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
    list_dirs: Vec<PathBuf>,
}

pub(super) fn run(
    executable: &Path,
    arguments: &[OsString],
    scope: &super::scope::Scope,
    temp_dir: &Path,
) -> Result<i32, String> {
    if !Path::new(SANDBOX_EXEC).is_file() {
        return Err(super::launch::sandbox_error());
    }
    let roots = sandbox_roots(scope, temp_dir);
    let xcrun_rule = if scope.mode == super::scope::Mode::Workspace {
        let rule = xcrun_cache_rule();
        if rule.is_some() {
            super::super::shell_diagnostics::clear_xcrun_failure();
        } else {
            super::super::shell_diagnostics::record_xcrun_failure();
        }
        rule
    } else {
        None
    };
    let mut command = std::process::Command::new(SANDBOX_EXEC);
    add_parameters(&mut command, "BEAVER_RW_DIR", &roots.write_dirs);
    add_parameters(&mut command, "BEAVER_RW_FILE", &roots.write_files);
    add_parameters(&mut command, "BEAVER_RO_DIR", &roots.read_dirs);
    add_parameters(&mut command, "BEAVER_RO_FILE", &roots.read_files);
    add_parameters(&mut command, "BEAVER_LIST_DIR", &roots.list_dirs);
    command
        .arg("-p")
        .arg(policy(&roots, scope.mode, xcrun_rule.as_deref()))
        .arg(executable)
        .args(arguments);
    let error = command.exec();
    Err(error.to_string())
}

fn sandbox_roots(scope: &super::scope::Scope, temp_dir: &Path) -> SandboxRoots {
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
    super::super::shell_sandbox_diagnostics::record(
        temp_dir,
        tools.path_limit_reached,
        tools.read_limit_reached,
        tools.write_limit_reached,
        tools.cache_unavailable,
        false,
    );
    if scope.mode == super::scope::Mode::ProfileCapture {
        return SandboxRoots {
            write_dirs: vec![temp_dir.to_path_buf()],
            write_files: Vec::new(),
            read_dirs: scope
                .roots
                .iter()
                .cloned()
                .chain(scope.read_dirs.iter().cloned())
                .chain(tools.read_dirs)
                .collect(),
            read_files: scope
                .read_files
                .iter()
                .cloned()
                .chain(tools.read_files)
                .collect(),
            list_dirs: tools.list_dirs,
        };
    }
    SandboxRoots {
        write_dirs: scope
            .roots
            .iter()
            .cloned()
            .chain(std::iter::once(temp_dir.to_path_buf()))
            .chain(tools.write_dirs)
            .collect(),
        write_files: tools.write_files,
        read_dirs: tools.read_dirs,
        read_files: tools.read_files,
        list_dirs: tools.list_dirs,
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

fn policy(
    roots: &SandboxRoots,
    mode: super::scope::Mode,
    xcrun_rule: Option<&str>,
) -> String {
    let mut policy = include_str!("macos_seatbelt_base.sbpl").to_string();
    policy.push_str(include_str!("macos_platform.sbpl"));
    if mode == super::scope::Mode::Workspace {
        policy.push_str(include_str!("macos_base.sbpl"));
        if let Some(rule) = xcrun_rule {
            policy.push_str(rule);
        }
    }
    append_rules(&mut policy, "BEAVER_RW_DIR", roots.write_dirs.len(), true, false);
    append_rules(&mut policy, "BEAVER_RW_FILE", roots.write_files.len(), true, true);
    append_rules(&mut policy, "BEAVER_RO_DIR", roots.read_dirs.len(), false, false);
    append_rules(&mut policy, "BEAVER_RO_FILE", roots.read_files.len(), false, true);
    append_list_rules(&mut policy, roots.list_dirs.len());
    policy
}

fn append_list_rules(policy: &mut String, count: usize) {
    for index in 0..count {
        policy.push_str(&format!(
            "(allow file-read-data file-read-metadata file-test-existence (literal (param \"BEAVER_LIST_DIR_{index}\")))\n"
        ));
    }
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
