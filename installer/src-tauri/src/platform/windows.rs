const MAX_DESTINATION_UNITS: usize = 1_024;
#[cfg(any(target_os = "windows", test))]
const DRIVE_FIXED: u32 = 3;

#[cfg(any(target_os = "windows", test))]
pub(crate) const fn fixed_drive_type(value: u32) -> bool {
    value == DRIVE_FIXED
}

#[cfg(target_os = "windows")]
use crate::error::InstallerError;
#[cfg(target_os = "windows")]
use std::path::{Path, PathBuf};

pub fn valid_destination_text(value: &str) -> bool {
    if value.encode_utf16().count() > MAX_DESTINATION_UNITS
        || value.chars().any(char::is_control)
        || value.starts_with("\\\\")
    {
        return false;
    }
    let bytes = value.as_bytes();
    if bytes.len() < 4
        || !bytes[0].is_ascii_alphabetic()
        || bytes[1] != b':'
        || !matches!(bytes[2], b'\\' | b'/')
    {
        return false;
    }
    let rest = &value[3..];
    if rest.contains(['"', '*', '?', '<', '>', '|', ':']) {
        return false;
    }
    rest.split(['\\', '/'])
        .filter(|segment| !segment.is_empty())
        .all(valid_segment)
}

fn valid_segment(segment: &str) -> bool {
    if segment == "." || segment == ".." || segment.ends_with(['.', ' ']) {
        return false;
    }
    let stem = segment
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !(stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
pub struct ValidatedDestination {
    raw: PathBuf,
    anchor: PathBuf,
    canonical_anchor: PathBuf,
}

#[cfg(target_os = "windows")]
pub fn validate_destination(value: &str) -> Result<ValidatedDestination, InstallerError> {
    if !valid_destination_text(value) {
        return Err(InstallerError::InstallFailed);
    }
    let raw = PathBuf::from(value);
    if !fixed_drive(&raw) {
        return Err(InstallerError::InstallFailed);
    }
    let (anchor, canonical_anchor) = validate_existing_chain(&raw)?;
    Ok(ValidatedDestination {
        raw,
        anchor,
        canonical_anchor,
    })
}

#[cfg(target_os = "windows")]
pub fn install_nsis(
    asset: &Path,
    work_dir: &Path,
    destination: &ValidatedDestination,
    before_spawn: impl FnOnce() -> Result<(), InstallerError>,
) -> Result<PathBuf, InstallerError> {
    use std::process::{Command, Stdio};

    let asset = validate_asset(asset, work_dir)?;
    revalidate(destination)?;
    before_spawn()?;
    let destination_argument = format!("/D={}", destination.raw.display());
    let status = Command::new(asset)
        .args(["/S", destination_argument.as_str()])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|_| InstallerError::InstallFailed)?;
    if !status.success() {
        return Err(InstallerError::InstallFailed);
    }
    validate_installed_executable(&destination.raw)
}

#[cfg(target_os = "windows")]
fn revalidate(destination: &ValidatedDestination) -> Result<(), InstallerError> {
    if !fixed_drive(&destination.raw) {
        return Err(InstallerError::InstallFailed);
    }
    let (anchor, canonical) = validate_existing_chain(&destination.raw)?;
    if anchor != destination.anchor || canonical != destination.canonical_anchor {
        return Err(InstallerError::InstallFailed);
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn fixed_drive(path: &Path) -> bool {
    use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;

    let Some(letter) = path.to_string_lossy().encode_utf16().next() else {
        return false;
    };
    let root = [letter, b':' as u16, b'\\' as u16, 0];
    fixed_drive_type(unsafe { GetDriveTypeW(root.as_ptr()) })
}

#[cfg(target_os = "windows")]
fn validate_existing_chain(path: &Path) -> Result<(PathBuf, PathBuf), InstallerError> {
    use std::os::windows::fs::MetadataExt;

    let mut current = Some(path);
    let mut nearest = None;
    while let Some(candidate) = current {
        match std::fs::symlink_metadata(candidate) {
            Ok(metadata) => {
                if !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
                    return Err(InstallerError::InstallFailed);
                }
                let canonical = candidate
                    .canonicalize()
                    .map_err(|_| InstallerError::InstallFailed)?;
                if nearest.is_none() {
                    nearest = Some((candidate.to_path_buf(), canonical));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(InstallerError::InstallFailed),
        }
        current = candidate.parent();
    }
    nearest.ok_or(InstallerError::InstallFailed)
}

#[cfg(target_os = "windows")]
fn validate_asset(asset: &Path, work_dir: &Path) -> Result<PathBuf, InstallerError> {
    use std::os::windows::fs::MetadataExt;

    let root = work_dir
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    let metadata = std::fs::symlink_metadata(asset).map_err(|_| InstallerError::InstallFailed)?;
    let canonical = asset
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    if !metadata.is_file()
        || metadata.file_attributes() & 0x400 != 0
        || canonical.parent() != Some(root.as_path())
        || canonical.extension().and_then(|value| value.to_str()) != Some("exe")
    {
        return Err(InstallerError::InstallFailed);
    }
    Ok(canonical)
}

#[cfg(target_os = "windows")]
pub fn validate_installed_executable(destination: &Path) -> Result<PathBuf, InstallerError> {
    use std::os::windows::fs::MetadataExt;

    let root = destination
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    let executable = root.join("cl-go-dash.exe");
    let metadata =
        std::fs::symlink_metadata(&executable).map_err(|_| InstallerError::InstallFailed)?;
    let canonical = executable
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    if !metadata.is_file()
        || metadata.file_attributes() & 0x400 != 0
        || canonical.parent() != Some(root.as_path())
    {
        return Err(InstallerError::InstallFailed);
    }
    Ok(canonical)
}
