use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;

const MAX_PRIVATE_ARG_UTF16: usize = 2_048;
const CREATE_PROCESS_UTF16_LIMIT: usize = 32_767;
const CEF_ADMISSION_PREFIX: &str = "--beaver-cef-admission=";
const CEF_ADMISSION_BARE: &str = "--beaver-cef-admission";
const CEF_PROCESS_TYPE_PREFIX: &str = "--type=";
const SHELL_SANDBOX_SWITCH: &str = "--beaver-shell-sandbox";

#[derive(Debug)]
pub(crate) enum BootstrapRole {
    Parent,
    CefHelper(zeroize::Zeroizing<String>),
    ShellSandbox,
}

pub(crate) fn classify_bootstrap(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<BootstrapRole, ()> {
    let mut arguments = arguments.into_iter();
    let _executable = arguments.next().ok_or(())?;
    let Some(first) = arguments.next() else {
        return Ok(BootstrapRole::Parent);
    };
    if matches_ascii(&first, SHELL_SANDBOX_SWITCH) {
        validate_shell_header(arguments)?;
        return Ok(BootstrapRole::ShellSandbox);
    }

    let mut process_type = 0_u8;
    let mut marker: Option<zeroize::Zeroizing<String>> = None;
    for argument in std::iter::once(first).chain(arguments) {
        if matches_ascii(&argument, SHELL_SANDBOX_SWITCH)
            || matches_ascii(&argument, CEF_ADMISSION_BARE)
        {
            return Err(());
        }
        if let Some(value) = bounded_private_suffix(&argument, CEF_ADMISSION_PREFIX) {
            let value = value?;
            if value.is_empty() || marker.is_some() {
                return Err(());
            }
            marker = Some(zeroize::Zeroizing::new(
                String::from_utf16(&value).map_err(|_| ())?,
            ));
        } else if let Some(value) = bounded_private_suffix(&argument, CEF_PROCESS_TYPE_PREFIX) {
            let value = value?;
            if value.is_empty() || process_type != 0 {
                return Err(());
            }
            process_type = 1;
        }
    }
    match (process_type, marker) {
        (0, None) => Ok(BootstrapRole::Parent),
        (1, Some(marker)) => Ok(BootstrapRole::CefHelper(marker)),
        _ => Err(()),
    }
}

fn validate_shell_header(arguments: impl IntoIterator<Item = OsString>) -> Result<(), ()> {
    for argument in arguments {
        if matches_ascii(&argument, "--") {
            return Ok(());
        }
        if matches_ascii(&argument, SHELL_SANDBOX_SWITCH)
            || matches_ascii(&argument, CEF_ADMISSION_BARE)
            || bounded_private_suffix(&argument, CEF_ADMISSION_PREFIX).is_some()
            || bounded_private_suffix(&argument, CEF_PROCESS_TYPE_PREFIX).is_some()
        {
            return Err(());
        }
    }
    Err(())
}

pub(crate) fn bootstrap_arguments(
    executable: &OsStr,
    forwarded: impl IntoIterator<Item = OsString>,
) -> Result<Vec<OsString>, ()> {
    let mut encoded_len = encoded_argument_len(executable, true)?
        .checked_add(1)
        .ok_or(())?;
    let mut result = Vec::new();
    for argument in forwarded {
        if is_module_switch(&argument) {
            return Err(());
        }
        encoded_len = encoded_len
            .checked_add(1)
            .and_then(|length| {
                encoded_argument_len(&argument, false)
                    .ok()?
                    .checked_add(length)
            })
            .ok_or(())?;
        if encoded_len > CREATE_PROCESS_UTF16_LIMIT {
            return Err(());
        }
        result.push(argument);
    }
    Ok(result)
}

fn is_module_switch(argument: &OsStr) -> bool {
    matches_ascii(argument, "--module") || bounded_private_suffix(argument, "--module=").is_some()
}

fn matches_ascii(value: &OsStr, expected: &str) -> bool {
    let mut wide = value.encode_wide();
    expected.bytes().all(|byte| {
        wide.next()
            .is_some_and(|unit| ascii_units_equal(unit, byte))
    }) && wide.next().is_none()
}

fn bounded_private_suffix(value: &OsStr, prefix: &str) -> Option<Result<Vec<u16>, ()>> {
    let mut wide = value.encode_wide();
    for byte in prefix.bytes() {
        let unit = wide.next()?;
        if !ascii_units_equal(unit, byte) {
            return None;
        }
    }
    let suffix = wide
        .by_ref()
        .take(MAX_PRIVATE_ARG_UTF16 + 1)
        .collect::<Vec<_>>();
    Some(
        (suffix.len() <= MAX_PRIVATE_ARG_UTF16)
            .then_some(suffix)
            .ok_or(()),
    )
}

fn ascii_units_equal(unit: u16, byte: u8) -> bool {
    let lowered_unit = if (b'A' as u16..=b'Z' as u16).contains(&unit) {
        unit + u16::from(b'a' - b'A')
    } else {
        unit
    };
    lowered_unit == u16::from(byte.to_ascii_lowercase())
}

fn encoded_argument_len(value: &OsStr, force_quotes: bool) -> Result<usize, ()> {
    let quote = force_quotes
        || value.encode_wide().next().is_none()
        || value
            .encode_wide()
            .any(|unit| unit == b' ' as u16 || unit == b'\t' as u16);
    let mut length = usize::from(quote);
    let mut slashes = 0_usize;
    for unit in value.encode_wide() {
        if unit == b'\\' as u16 {
            slashes = slashes.checked_add(1).ok_or(())?;
            continue;
        }
        let emitted = if unit == b'"' as u16 {
            slashes
                .checked_mul(2)
                .and_then(|value| value.checked_add(2))
        } else {
            slashes.checked_add(1)
        }
        .ok_or(())?;
        length = length.checked_add(emitted).ok_or(())?;
        slashes = 0;
    }
    let trailing = if quote {
        slashes
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
    } else {
        Some(slashes)
    }
    .ok_or(())?;
    length.checked_add(trailing).ok_or(())
}

#[cfg(test)]
#[path = "windows_entry_arguments_tests.rs"]
mod tests;
