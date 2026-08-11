use super::super::CefUnavailableCategory;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use zeroize::Zeroizing;

const ADMISSION_PREFIX: &[u8] = b"--beaver-cef-admission=";
const TYPE_PREFIX: &[u8] = b"--type=";
const MAX_PRIVATE_ARGUMENT_BYTES: usize = 2_048;

pub(in crate::services::browser) fn parse_helper_marker(
) -> Result<Zeroizing<String>, CefUnavailableCategory> {
    parse_helper_marker_from(std::env::args_os())
}

fn parse_helper_marker_from(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Zeroizing<String>, CefUnavailableCategory> {
    let mut marker = None;
    let mut process_type = false;
    for raw in arguments {
        if let Some(found) = bounded_private_suffix(&raw, ADMISSION_PREFIX) {
            let found = found?;
            if found.is_empty() || marker.is_some() {
                return Err(CefUnavailableCategory::Admission);
            }
            marker = Some(Zeroizing::new(
                std::str::from_utf8(found)
                    .map_err(|_| CefUnavailableCategory::Admission)?
                    .to_string(),
            ));
        } else if let Some(found) = bounded_private_suffix(&raw, TYPE_PREFIX) {
            let found = found?;
            if found.is_empty() || process_type {
                return Err(CefUnavailableCategory::Admission);
            }
            process_type = true;
        }
    }
    match (process_type, marker) {
        (true, Some(marker)) => Ok(marker),
        _ => Err(CefUnavailableCategory::Admission),
    }
}

fn bounded_private_suffix<'a>(
    value: &'a OsStr,
    prefix: &[u8],
) -> Option<Result<&'a [u8], CefUnavailableCategory>> {
    let bytes = value.as_bytes();
    let candidate = bytes.get(..prefix.len())?;
    if !candidate.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let suffix = &bytes[prefix.len()..];
    Some(
        (suffix.len() <= MAX_PRIVATE_ARGUMENT_BYTES)
            .then_some(suffix)
            .ok_or(CefUnavailableCategory::Admission),
    )
}

#[cfg(test)]
#[path = "arguments_tests.rs"]
mod tests;
