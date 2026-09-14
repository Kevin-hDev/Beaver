use crate::error::InstallerError;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Component, PathBuf};

const MAX_ASSET_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const KEYS: [&str; 6] = [
    "--run-id",
    "--work-dir",
    "--version",
    "--app-asset-name",
    "--app-asset-size",
    "--app-asset-sha256",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedRelease {
    pub version: String,
    pub app_asset_name: String,
    pub app_asset_size: u64,
    pub app_asset_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchContext {
    pub run_id: String,
    pub work_dir: PathBuf,
    pub release: PinnedRelease,
}

impl LaunchContext {
    pub fn parse_from<I, S>(args: I) -> Result<Self, InstallerError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut args = args.into_iter().map(Into::into);
        args.next().ok_or(InstallerError::InvalidLaunch)?;
        let mut values = BTreeMap::new();
        while let Some(raw_key) = args.next() {
            let key = raw_key
                .into_string()
                .map_err(|_| InstallerError::InvalidLaunch)?;
            if !KEYS.contains(&key.as_str()) || values.contains_key(key.as_str()) {
                return Err(InstallerError::InvalidLaunch);
            }
            let value = args
                .next()
                .ok_or(InstallerError::InvalidLaunch)?
                .into_string()
                .map_err(|_| InstallerError::InvalidLaunch)?;
            if value.is_empty() || value.chars().any(char::is_control) {
                return Err(InstallerError::InvalidLaunch);
            }
            values.insert(key, value);
        }
        if values.len() != KEYS.len() {
            return Err(InstallerError::InvalidLaunch);
        }

        let run_id = take(&mut values, "--run-id")?;
        let work_dir = PathBuf::from(take(&mut values, "--work-dir")?);
        let version = take(&mut values, "--version")?;
        let app_asset_name = take(&mut values, "--app-asset-name")?;
        let app_asset_size = take(&mut values, "--app-asset-size")?
            .parse()
            .map_err(|_| InstallerError::InvalidLaunch)?;
        let app_asset_sha256 = take(&mut values, "--app-asset-sha256")?;

        if !valid_hex(&run_id, 32)
            || !valid_work_dir(&work_dir)
            || !valid_version(&version)
            || !(1..=MAX_ASSET_BYTES).contains(&app_asset_size)
            || !valid_hex(&app_asset_sha256, 64)
            || expected_asset_name(&version).as_deref() != Some(app_asset_name.as_str())
        {
            return Err(InstallerError::InvalidLaunch);
        }

        Ok(Self {
            run_id,
            work_dir,
            release: PinnedRelease {
                version,
                app_asset_name,
                app_asset_size,
                app_asset_sha256,
            },
        })
    }
}

fn take(values: &mut BTreeMap<String, String>, key: &str) -> Result<String, InstallerError> {
    values.remove(key).ok_or(InstallerError::InvalidLaunch)
}

fn valid_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_version(value: &str) -> bool {
    let mut parts = value.split('.');
    let valid = (0..3).all(|_| {
        parts.next().is_some_and(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == "0" || !part.starts_with('0'))
        })
    });
    valid && parts.next().is_none()
}

fn valid_work_dir(path: &std::path::Path) -> bool {
    path.is_absolute()
        && path
            .components()
            .all(|component| !matches!(component, Component::ParentDir))
}

#[cfg(target_os = "macos")]
fn expected_asset_name(version: &str) -> Option<String> {
    Some(format!("Beaver_{version}_aarch64.dmg"))
}

#[cfg(target_os = "windows")]
fn expected_asset_name(version: &str) -> Option<String> {
    Some(format!("Beaver_{version}_x64-setup.exe"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn expected_asset_name(_version: &str) -> Option<String> {
    None
}
