use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::Path;

use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_BINARY};
use winreg::{RegKey, RegValue};

use super::super::{ExactEntryState, MigrationError};

const RUN_KEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
const APPROVED_KEY: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";
const APPROVED: [u8; 12] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub(super) fn state(name: &str, executable: &Path) -> Result<ExactEntryState, MigrationError> {
    let actual: Option<OsString> =
        match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN_KEY, KEY_READ) {
            Ok(key) => match key.get_value(name) {
                Ok(value) => Some(value),
                Err(error) if error.kind() == ErrorKind::NotFound => None,
                Err(_) => return Err(MigrationError::State),
            },
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(_) => return Err(MigrationError::State),
        };
    let approval = approval(name)?;
    Ok(if actual.is_none() && approval.is_none() {
        ExactEntryState::Absent
    } else if actual == Some(command(executable))
        && approval.as_deref().is_none_or(|v| v == APPROVED)
    {
        ExactEntryState::Exact
    } else {
        ExactEntryState::Stale
    })
}

pub(super) fn install(name: &str, executable: &Path) -> Result<(), MigrationError> {
    RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey(RUN_KEY)
        .and_then(|(key, _)| key.set_value(name, &command(executable)))
        .map_err(|_| MigrationError::State)?;
    if let Ok(key) =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(APPROVED_KEY, KEY_SET_VALUE)
    {
        key.set_raw_value(
            name,
            &RegValue {
                vtype: REG_BINARY,
                bytes: APPROVED.to_vec(),
            },
        )
        .map_err(|_| MigrationError::State)?;
    }
    Ok(())
}

pub(super) fn remove(name: &str) -> Result<(), MigrationError> {
    match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE) {
        Ok(key) => delete_value(&key, name)?,
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(_) => return Err(MigrationError::State),
    }
    if let Ok(key) =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(APPROVED_KEY, KEY_SET_VALUE)
    {
        delete_value(&key, name)?;
    }
    Ok(())
}

fn approval(name: &str) -> Result<Option<Vec<u8>>, MigrationError> {
    let key = match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(APPROVED_KEY, KEY_READ)
    {
        Ok(key) => key,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(MigrationError::State),
    };
    match key.get_raw_value(name) {
        Ok(value) if value.vtype == REG_BINARY => Ok(Some(value.bytes)),
        Ok(_) => Ok(Some(Vec::new())),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(_) => Err(MigrationError::State),
    }
}

fn delete_value(key: &RegKey, name: &str) -> Result<(), MigrationError> {
    match key.delete_value(name) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(_) => Err(MigrationError::State),
    }
}

fn command(executable: &Path) -> OsString {
    let mut command = OsString::from("\"");
    command.push(executable.as_os_str());
    command.push("\" ");
    command.push(crate::app_events::AUTOSTART_ARG);
    command
}
