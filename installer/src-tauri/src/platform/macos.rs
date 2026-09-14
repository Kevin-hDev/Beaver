use super::macos_recovery::managed_sibling_kind;
use crate::error::InstallerError;
use std::ffi::{CString, OsStr};
use std::fs::{self, OpenOptions};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const APPLICATIONS: &str = "/Applications";
const MAX_PLIST_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ValidatedDestination {
    root: PathBuf,
    needs_authorization: bool,
}

impl ValidatedDestination {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn needs_authorization(&self) -> bool {
        self.needs_authorization
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedBundle {
    root: PathBuf,
    executable: PathBuf,
}

impl ValidatedBundle {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }
}

pub fn validate_destination(path: &Path) -> Result<ValidatedDestination, InstallerError> {
    if !path.is_absolute()
        || path.components().any(|part| part == Component::ParentDir)
        || path.as_os_str().as_bytes().contains(&0)
    {
        return Err(InstallerError::InstallFailed);
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| InstallerError::InstallFailed)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(InstallerError::InstallFailed);
    }
    let root = path
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    if !local_volume(&root) {
        return Err(InstallerError::InstallFailed);
    }
    let applications = Path::new(APPLICATIONS);
    let needs_authorization = if root == applications {
        !writable_probe(&root)
    } else {
        if !writable_probe(&root) {
            return Err(InstallerError::InstallFailed);
        }
        false
    };
    let destination = ValidatedDestination {
        root,
        needs_authorization,
    };
    validate_target(&destination)?;
    Ok(destination)
}

pub fn validate_target(destination: &ValidatedDestination) -> Result<PathBuf, InstallerError> {
    let target = destination.root.join("Beaver.app");
    match fs::symlink_metadata(&target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(target),
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(target),
        _ => Err(InstallerError::InstallFailed),
    }
}

pub fn validate_bundle(path: &Path, mount_root: &Path) -> Result<ValidatedBundle, InstallerError> {
    let mount = mount_root
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    if path.file_name() != Some(OsStr::new("Beaver.app")) {
        return Err(InstallerError::InstallFailed);
    }
    let validated = validate_bundle_contents(path)?;
    let root = validated.root();
    if root.parent() != Some(mount.as_path()) {
        return Err(InstallerError::InstallFailed);
    }
    Ok(validated)
}

pub fn validate_staged_bundle(
    path: &Path,
    destination: &ValidatedDestination,
) -> Result<ValidatedBundle, InstallerError> {
    if !valid_destination_bundle_name(path.file_name().and_then(OsStr::to_str)) {
        return Err(InstallerError::InstallFailed);
    }
    let validated = validate_bundle_contents(path)?;
    (validated.root.parent() == Some(destination.root.as_path()))
        .then_some(validated)
        .ok_or(InstallerError::InstallFailed)
}

fn valid_destination_bundle_name(name: Option<&str>) -> bool {
    let Some(name) = name else {
        return false;
    };
    if name == "Beaver.app" {
        return true;
    }
    managed_sibling_kind(name).is_some()
}

pub fn installed_bundle(destination: &Path) -> Result<ValidatedBundle, InstallerError> {
    let destination = validate_destination(destination)?;
    validate_staged_bundle(&destination.root.join("Beaver.app"), &destination)
}

pub fn bundle_version(bundle: &ValidatedBundle) -> Result<String, InstallerError> {
    plist_value(
        &bundle.root.join("Contents/Info.plist"),
        "CFBundleShortVersionString",
    )
}

fn validate_bundle_contents(path: &Path) -> Result<ValidatedBundle, InstallerError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| InstallerError::InstallFailed)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(InstallerError::InstallFailed);
    }
    let root = path
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    let plist = root.join("Contents/Info.plist");
    validate_regular_file(&plist, &root, Some(MAX_PLIST_BYTES))?;
    if plist_value(&plist, "CFBundleIdentifier")? != "com.clgo.dash"
        || plist_value(&plist, "CFBundleExecutable")? != "cl-go-dash"
    {
        return Err(InstallerError::InstallFailed);
    }
    let executable = validate_regular_file(&root.join("Contents/MacOS/cl-go-dash"), &root, None)?;
    Ok(ValidatedBundle { root, executable })
}

fn validate_regular_file(
    path: &Path,
    root: &Path,
    max_size: Option<u64>,
) -> Result<PathBuf, InstallerError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| InstallerError::InstallFailed)?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || max_size.is_some_and(|max| metadata.len() == 0 || metadata.len() > max)
    {
        return Err(InstallerError::InstallFailed);
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    canonical
        .starts_with(root)
        .then_some(canonical)
        .ok_or(InstallerError::InstallFailed)
}

fn plist_value(path: &Path, key: &str) -> Result<String, InstallerError> {
    let output = Command::new("/usr/bin/plutil")
        .args(["-extract", key, "raw", "-o", "-"])
        .arg(path)
        .env_clear()
        .output()
        .map_err(|_| InstallerError::InstallFailed)?;
    if !output.status.success() || output.stdout.len() > MAX_PLIST_BYTES as usize {
        return Err(InstallerError::InstallFailed);
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|_| InstallerError::InstallFailed)
}

fn writable_probe(root: &Path) -> bool {
    let mut random = [0_u8; 16];
    rand::fill(&mut random);
    let path = root.join(format!(".beaver-write-probe-{}", hex::encode(random)));
    let mut options = OpenOptions::new();
    use std::os::unix::fs::OpenOptionsExt;
    options.write(true).create_new(true).mode(0o600);
    match options.open(&path) {
        Ok(file) => {
            drop(file);
            fs::remove_file(path).is_ok()
        }
        Err(_) => false,
    }
}

fn local_volume(path: &Path) -> bool {
    let Ok(path) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    let mut info = std::mem::MaybeUninit::<libc::statfs>::uninit();
    let result = unsafe { libc::statfs(path.as_ptr(), info.as_mut_ptr()) };
    result == 0 && unsafe { info.assume_init() }.f_flags & libc::MNT_LOCAL as u32 != 0
}
