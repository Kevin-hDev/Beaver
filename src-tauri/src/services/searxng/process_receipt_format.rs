use serde::Deserialize;

use super::SearxngProcessReceipt;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearxngProcessReceiptWire {
    schema_version: u8,
    pid: u32,
    native_start_time: u64,
    native_scope: u64,
    executable_high: u64,
    executable_low: u64,
    #[serde(default)]
    pending: bool,
}

pub(super) fn parse(bytes: &[u8]) -> Result<SearxngProcessReceipt, ()> {
    let wire = serde_json::from_slice::<SearxngProcessReceiptWire>(bytes).map_err(|_| ())?;
    let receipt = SearxngProcessReceipt {
        schema_version: wire.schema_version,
        pid: wire.pid,
        native_start_time: wire.native_start_time,
        native_scope: wire.native_scope,
        executable_high: wire.executable_high,
        executable_low: wire.executable_low,
        pending: wire.pending,
    };
    receipt.valid().then_some(receipt).ok_or(())
}

pub(super) fn legacy_numeric(bytes: &[u8]) -> bool {
    bytes.iter().any(u8::is_ascii_digit)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_digit() || byte.is_ascii_whitespace())
}
