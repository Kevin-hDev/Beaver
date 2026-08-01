use subtle::{Choice, ConstantTimeEq};
use zeroize::{Zeroize, Zeroizing};

pub(super) const MAX_REQUEST_BYTES: usize = 8 * 1024;
const MAX_CODE_BYTES: usize = 4 * 1024;
const STATE_HEX_CHARS: usize = 32;
const MAX_QUERY_PAIRS: usize = 8;

#[derive(Debug)]
pub struct CallbackResult {
    pub code: Zeroizing<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CallbackError {
    Invalid,
    Refused,
}

pub(super) fn parse_callback_bytes(
    request: &[u8],
    expected_state: &str,
) -> Result<CallbackResult, CallbackError> {
    if request.is_empty() || request.len() > MAX_REQUEST_BYTES {
        return Err(CallbackError::Invalid);
    }
    let request = std::str::from_utf8(request).map_err(|_| CallbackError::Invalid)?;
    let first_line = request.split("\r\n").next().ok_or(CallbackError::Invalid)?;
    let mut parts = first_line.split_whitespace();
    if parts.next() != Some("GET") {
        return Err(CallbackError::Invalid);
    }
    let target = parts.next().ok_or(CallbackError::Invalid)?;
    if parts.next() != Some("HTTP/1.1") || parts.next().is_some() {
        return Err(CallbackError::Invalid);
    }
    parse_target(target, expected_state)
}

fn parse_target(target: &str, expected_state: &str) -> Result<CallbackResult, CallbackError> {
    let (path, query) = target.split_once('?').ok_or(CallbackError::Invalid)?;
    if path != "/auth/callback" {
        return Err(CallbackError::Invalid);
    }
    let mut code = None;
    let mut state = None;
    let mut refused = false;
    let mut count = 0_usize;
    for pair in query.split('&') {
        count += 1;
        if count > MAX_QUERY_PAIRS {
            return Err(CallbackError::Invalid);
        }
        let (key, value) = pair.split_once('=').ok_or(CallbackError::Invalid)?;
        match key {
            "code" if code.is_none() => code = Some(value),
            "state" if state.is_none() => state = Some(value),
            "error" if !refused && !value.is_empty() => refused = true,
            "code" | "state" | "error" => return Err(CallbackError::Invalid),
            _ => {}
        }
    }
    let state = state.ok_or(CallbackError::Invalid)?;
    if !constant_time_state_eq(state, expected_state) {
        return Err(CallbackError::Invalid);
    }
    if refused {
        return Err(CallbackError::Refused);
    }
    let encoded = code.ok_or(CallbackError::Invalid)?;
    if encoded.is_empty() || encoded.len() > MAX_CODE_BYTES {
        return Err(CallbackError::Invalid);
    }
    let decoded = urlencoding::decode(encoded).map_err(|_| CallbackError::Invalid)?;
    if decoded.is_empty() || decoded.len() > MAX_CODE_BYTES {
        return Err(CallbackError::Invalid);
    }
    Ok(CallbackResult {
        code: Zeroizing::new(decoded.into_owned()),
    })
}

pub(super) fn validate_state(state: &str) -> Result<(), String> {
    if state.len() == STATE_HEX_CHARS && state.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("état OAuth invalide".to_string())
    }
}

fn constant_time_state_eq(actual: &str, expected: &str) -> bool {
    let mut actual_fixed = [0_u8; STATE_HEX_CHARS];
    let mut expected_fixed = [0_u8; STATE_HEX_CHARS];
    let actual_valid = copy_state(actual, &mut actual_fixed);
    let expected_valid = copy_state(expected, &mut expected_fixed);
    let validity = Choice::from((actual_valid | expected_valid) ^ 1);
    let equal = bool::from(actual_fixed.ct_eq(&expected_fixed) & validity);
    actual_fixed.zeroize();
    expected_fixed.zeroize();
    equal
}

fn copy_state(input: &str, output: &mut [u8; STATE_HEX_CHARS]) -> u8 {
    let bytes = input.as_bytes();
    let valid = u8::from(
        bytes.len() == STATE_HEX_CHARS && bytes.iter().all(|byte| byte.is_ascii_hexdigit()),
    );
    for (index, byte) in bytes.iter().take(STATE_HEX_CHARS).enumerate() {
        output[index] = *byte;
    }
    valid ^ 1
}

#[cfg(test)]
#[path = "callback_tests.rs"]
mod tests;
