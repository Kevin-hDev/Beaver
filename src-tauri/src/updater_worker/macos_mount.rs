use rand::RngCore;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use super::command::{run_bounded_output, run_status, CommandSpec};
use super::WorkerError;

const MAX_ATTACH_OUTPUT: usize = 64 * 1024;
const MAX_ENTITIES: usize = 16;
const MAX_MOUNT_ATTEMPTS: usize = 8;
const COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) struct MountedDmg {
    mount_point: PathBuf,
    mounted: bool,
}

impl MountedDmg {
    pub(crate) fn attach(asset: &Path) -> Result<Self, WorkerError> {
        let mount_point = create_mount_point(&std::env::temp_dir())?;
        let output = match run_bounded_output(
            &attach_spec(asset, &mount_point),
            COMMAND_TIMEOUT,
            MAX_ATTACH_OUTPUT,
        ) {
            Ok(output) => output,
            Err(error) => {
                let _ = detach_mount(&mount_point);
                let _ = std::fs::remove_dir(&mount_point);
                return Err(error);
            }
        };
        if parse_mount_point(&output, &mount_point).is_err() {
            let _ = detach_mount(&mount_point);
            let _ = std::fs::remove_dir(&mount_point);
            return Err(WorkerError);
        }
        Ok(Self {
            mount_point,
            mounted: true,
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.mount_point
    }

    pub(crate) fn detach(mut self) -> Result<(), WorkerError> {
        let result = detach_mount(&self.mount_point);
        if result.is_ok() {
            self.mounted = false;
            std::fs::remove_dir(&self.mount_point).map_err(|_| WorkerError)?;
        }
        result
    }
}

impl Drop for MountedDmg {
    fn drop(&mut self) {
        if self.mounted {
            let _ = detach_mount(&self.mount_point);
            let _ = std::fs::remove_dir(&self.mount_point);
        }
    }
}

pub(crate) fn attach_spec(asset: &Path, mount_point: &Path) -> CommandSpec {
    CommandSpec::new(
        "/usr/bin/hdiutil",
        vec![
            OsString::from("attach"),
            asset.as_os_str().to_owned(),
            OsString::from("-nobrowse"),
            OsString::from("-readonly"),
            OsString::from("-mountpoint"),
            mount_point.as_os_str().to_owned(),
            OsString::from("-plist"),
        ],
    )
}

fn detach_mount(mount_point: &Path) -> Result<(), WorkerError> {
    run_status(
        &CommandSpec::new(
            "/usr/bin/hdiutil",
            vec![OsString::from("detach"), mount_point.as_os_str().to_owned()],
        ),
        COMMAND_TIMEOUT,
    )
}

fn create_mount_point(temp_root: &Path) -> Result<PathBuf, WorkerError> {
    let root = std::fs::canonicalize(temp_root).map_err(|_| WorkerError)?;
    for _ in 0..MAX_MOUNT_ATTEMPTS {
        let mut random = [0_u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut random);
        let path = root.join(format!("cl-go-dash-mount-{}", hex::encode(random)));
        match std::fs::create_dir(&path) {
            Ok(()) => {
                if set_private_directory(&path).is_err() {
                    let _ = std::fs::remove_dir(&path);
                    return Err(WorkerError);
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(WorkerError),
        }
    }
    Err(WorkerError)
}

fn parse_mount_point(output: &[u8], expected: &Path) -> Result<(), WorkerError> {
    let value = plist::Value::from_reader(std::io::Cursor::new(output)).map_err(|_| WorkerError)?;
    let entities = value
        .as_dictionary()
        .and_then(|dictionary| dictionary.get("system-entities"))
        .and_then(plist::Value::as_array)
        .ok_or(WorkerError)?;
    if entities.is_empty() || entities.len() > MAX_ENTITIES {
        return Err(WorkerError);
    }
    let mut matches = 0_usize;
    for entity in entities {
        let Some(raw) = entity
            .as_dictionary()
            .and_then(|dictionary| dictionary.get("mount-point"))
            .and_then(plist::Value::as_string)
        else {
            continue;
        };
        let path = Path::new(raw);
        if !path.is_absolute()
            || path
                .components()
                .any(|component| component == Component::ParentDir)
        {
            return Err(WorkerError);
        }
        if std::fs::canonicalize(path).map_err(|_| WorkerError)? == expected {
            matches += 1;
        }
    }
    if matches == 1 {
        Ok(())
    } else {
        Err(WorkerError)
    }
}

#[cfg(unix)]
fn set_private_directory(path: &Path) -> Result<(), WorkerError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(|_| WorkerError)
}

#[cfg(not(unix))]
fn set_private_directory(_path: &Path) -> Result<(), WorkerError> {
    Ok(())
}

#[cfg(test)]
#[path = "macos_mount_tests.rs"]
mod tests;
