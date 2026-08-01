use regex::Regex;
use std::sync::LazyLock;

static RANGE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Z]+)(\d+):([A-Z]+)(\d+)$").unwrap());
const MAX_COLUMN_INDEX: usize = 16_383;
const MAX_ROW_NUMBER: usize = 1_048_576;

pub(super) fn parse(value: &str) -> Option<(usize, usize, usize, usize)> {
    let captures = RANGE_REGEX.captures(value)?;
    let column_start = column_index(&captures[1])?;
    let row_start = row_index(&captures[2])?;
    let column_end = column_index(&captures[3])?;
    let row_end = row_index(&captures[4])?;
    (row_start <= row_end && column_start <= column_end)
        .then_some((row_start, column_start, row_end, column_end))
}

pub(super) fn column_index(value: &str) -> Option<usize> {
    if value.is_empty() || value.len() > 3 || !value.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return None;
    }
    let one_based = value.bytes().try_fold(0usize, |index, byte| {
        index
            .checked_mul(26)?
            .checked_add(usize::from(byte - b'A' + 1))
    })?;
    let index = one_based.checked_sub(1)?;
    (index <= MAX_COLUMN_INDEX).then_some(index)
}

pub(super) fn row_index(value: &str) -> Option<usize> {
    let one_based = value.parse::<usize>().ok()?;
    if one_based == 0 || one_based > MAX_ROW_NUMBER {
        return None;
    }
    one_based.checked_sub(1)
}
