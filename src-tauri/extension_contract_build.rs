use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

const CONTRACT_PATH: &str = "resources/extension-host/contract.json";

pub fn generate() {
    println!("cargo:rerun-if-changed={CONTRACT_PATH}");
    let raw =
        std::fs::read_to_string(CONTRACT_PATH).expect("cannot read the Beaver extension contract");
    assert!(
        raw.len() <= 8_192,
        "Beaver extension contract exceeds its size limit"
    );
    let contract: Value =
        serde_json::from_str(&raw).expect("invalid Beaver extension contract JSON");
    let limits = object(&contract, "limits");
    let diagnostics = object(&contract, "diagnostics");
    let host_codes = codes(diagnostics, "hostCodes");
    let runtime_codes = codes(diagnostics, "runtimeCodes");
    validate_unique_codes(&host_codes, &runtime_codes);

    let mut output = String::new();
    for (json_name, rust_name) in [
        ("maxExtensions", "MAX_EXTENSIONS"),
        ("maxUserExtensions", "MAX_USER_EXTENSIONS"),
        ("maxTools", "MAX_TOOLS"),
        ("maxToolsPerExtension", "MAX_TOOLS_PER_EXTENSION"),
        ("maxEventsPerExtension", "MAX_EVENTS_PER_EXTENSION"),
        ("maxPendingRequests", "MAX_PENDING_REQUESTS"),
        ("maxInFlightRequests", "MAX_IN_FLIGHT_REQUESTS"),
        ("maxWorkingDirectoryChars", "MAX_WORKING_DIRECTORY_CHARS"),
        ("maxMessageBytes", "MAX_MESSAGE_BYTES"),
    ] {
        let value = limit(limits, json_name);
        output.push_str(&format!("pub const {rust_name}: usize = {value};\n"));
    }
    for code in host_codes.iter().chain(&runtime_codes) {
        output.push_str(&format!(
            "pub const {}: &str = {code:?};\n",
            code_constant(code)
        ));
    }
    output.push_str(&code_slice("HOST_DIAGNOSTIC_CODES", &host_codes));
    output.push_str(&code_slice("RUNTIME_DIAGNOSTIC_CODES", &runtime_codes));
    let all_codes = host_codes
        .into_iter()
        .chain(runtime_codes)
        .collect::<Vec<_>>();
    output.push_str("#[cfg(test)]\n");
    output.push_str(&code_slice("DIAGNOSTIC_CODES", &all_codes));

    let out_dir = std::env::var_os("OUT_DIR").expect("missing Cargo OUT_DIR");
    std::fs::write(Path::new(&out_dir).join("extension_contract.rs"), output)
        .expect("cannot generate the Beaver extension contract");
}

fn object<'a>(value: &'a Value, name: &str) -> &'a Map<String, Value> {
    value
        .get(name)
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing extension contract object: {name}"))
}

fn limit(limits: &Map<String, Value>, name: &str) -> usize {
    let value = limits
        .get(name)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("invalid extension contract limit: {name}"));
    assert!(
        (1..=16_777_216).contains(&value),
        "extension contract limit out of range: {name}"
    );
    usize::try_from(value).expect("extension contract limit exceeds usize")
}

fn codes(diagnostics: &Map<String, Value>, name: &str) -> Vec<String> {
    let values = diagnostics
        .get(name)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing extension diagnostic codes: {name}"));
    assert!(
        (1..=32).contains(&values.len()),
        "invalid extension diagnostic code count: {name}"
    );
    values
        .iter()
        .map(|value| {
            let code = value
                .as_str()
                .unwrap_or_else(|| panic!("invalid extension diagnostic code: {name}"));
            assert!(
                valid_code(code),
                "invalid extension diagnostic code: {code}"
            );
            code.to_string()
        })
        .collect()
}

fn validate_unique_codes(host: &[String], runtime: &[String]) {
    let mut unique = BTreeSet::new();
    for code in host.iter().chain(runtime) {
        assert!(
            unique.insert(code),
            "duplicate extension diagnostic code: {code}"
        );
    }
}

fn valid_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 64
        && code.as_bytes()[0].is_ascii_lowercase()
        && code
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn code_constant(code: &str) -> String {
    format!("DIAGNOSTIC_{}", code.to_ascii_uppercase())
}

fn code_slice(name: &str, codes: &[String]) -> String {
    let values = codes
        .iter()
        .map(|code| code_constant(code))
        .collect::<Vec<_>>()
        .join(", ");
    format!("pub const {name}: &[&str] = &[{values}];\n")
}
