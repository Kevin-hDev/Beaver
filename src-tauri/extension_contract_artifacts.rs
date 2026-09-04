use serde_json::{Map, Value};

pub fn render_typescript(contract: &Value) -> Result<String, String> {
    let methods = object(contract, "methods")?;
    let host_methods = array(methods, "hostToCore")?;
    let stable = method_names(host_methods, Some("stable"), Some("request"))?;
    let advanced = method_names(host_methods, Some("advanced"), Some("request"))?;
    let notifications = method_names(host_methods, None, Some("notification"))?;
    let mut output = String::from("// Generated from src-tauri/resources/extension-host/contract.json.\n// Do not edit by hand.\n\n");
    output.push_str(&format!(
        "export const EXTENSION_API_VERSION = {:?} as const;\n",
        string(contract, "apiVersion")?
    ));
    for (name, values) in [
        (
            "EXTENSION_CAPABILITIES",
            array_value(contract, "capabilities")?.to_vec(),
        ),
        (
            "OPTIONAL_EXTENSION_CAPABILITIES",
            array_value(contract, "optionalCapabilities")?.to_vec(),
        ),
        (
            "EXTENSION_CONTRIBUTION_TYPES",
            array_value(contract, "contributionTypes")?.to_vec(),
        ),
        (
            "EXTENSION_RESULT_BLOCK_TYPES",
            array_value(contract, "resultBlockTypes")?.to_vec(),
        ),
        (
            "EXTENSION_RESULT_FILE_PURPOSES",
            array_value(contract, "resultFilePurposes")?.to_vec(),
        ),
        (
            "EXTENSION_RESOURCE_TYPES",
            array_value(contract, "resourceTypes")?.to_vec(),
        ),
        (
            "CORE_TO_HOST_METHODS",
            array(methods, "coreToHost")?.to_vec(),
        ),
        ("STABLE_HOST_TO_CORE_REQUEST_METHODS", stable),
        ("ADVANCED_HOST_TO_CORE_REQUEST_METHODS", advanced),
        ("HOST_TO_CORE_NOTIFICATION_METHODS", notifications),
        (
            "EXTENSION_EVENTS",
            array_value(contract, "events")?.to_vec(),
        ),
        (
            "EXTENSION_HOST_STATES",
            array_value(contract, "hostStates")?.to_vec(),
        ),
        (
            "HOST_LOAD_STAGES",
            array_value(contract, "loadStages")?.to_vec(),
        ),
        (
            "EXTENSION_EFFECT_CLASSES",
            array_value(contract, "effectClasses")?.to_vec(),
        ),
        (
            "PROTOCOL_ERROR_REASONS",
            array(object(contract, "errors")?, "protocolReasons")?.to_vec(),
        ),
        (
            "EXTENSION_BACKEND_ERROR_CODES",
            array(object(contract, "errors")?, "backendCodes")?.to_vec(),
        ),
        (
            "HOST_DIAGNOSTIC_CODES",
            array(object(contract, "diagnostics")?, "hostCodes")?.to_vec(),
        ),
        (
            "RUNTIME_DIAGNOSTIC_CODES",
            array(object(contract, "diagnostics")?, "runtimeCodes")?.to_vec(),
        ),
    ] {
        output.push_str(&format!(
            "export const {name} = {} as const;\n",
            json(&Value::Array(values))?
        ));
    }
    output.push_str(&format!(
        "export const LIMITS = Object.freeze({} as const);\n",
        json(&Value::Object(object(contract, "limits")?.clone()))?
    ));
    output.push_str(&format!(
        "export const TIMEOUTS = Object.freeze({} as const);\n\n",
        json(&Value::Object(object(contract, "timeouts")?.clone()))?
    ));
    for (type_name, constant) in [
        ("ExtensionCapability", "EXTENSION_CAPABILITIES"),
        (
            "OptionalExtensionCapability",
            "OPTIONAL_EXTENSION_CAPABILITIES",
        ),
        ("ExtensionContributionType", "EXTENSION_CONTRIBUTION_TYPES"),
        ("ExtensionResultBlockType", "EXTENSION_RESULT_BLOCK_TYPES"),
        (
            "ExtensionResultFilePurpose",
            "EXTENSION_RESULT_FILE_PURPOSES",
        ),
        ("ExtensionResourceType", "EXTENSION_RESOURCE_TYPES"),
        ("CoreToHostMethod", "CORE_TO_HOST_METHODS"),
        (
            "StableHostToCoreRequestMethod",
            "STABLE_HOST_TO_CORE_REQUEST_METHODS",
        ),
        (
            "AdvancedHostToCoreRequestMethod",
            "ADVANCED_HOST_TO_CORE_REQUEST_METHODS",
        ),
        (
            "HostToCoreNotificationMethod",
            "HOST_TO_CORE_NOTIFICATION_METHODS",
        ),
        ("ExtensionEvent", "EXTENSION_EVENTS"),
        ("ExtensionHostState", "EXTENSION_HOST_STATES"),
        ("HostLoadStage", "HOST_LOAD_STAGES"),
        ("ExtensionEffectClass", "EXTENSION_EFFECT_CLASSES"),
        ("ExtensionProtocolErrorReason", "PROTOCOL_ERROR_REASONS"),
        ("ExtensionBackendErrorCode", "EXTENSION_BACKEND_ERROR_CODES"),
        ("HostDiagnosticCode", "HOST_DIAGNOSTIC_CODES"),
        ("RuntimeDiagnosticCode", "RUNTIME_DIAGNOSTIC_CODES"),
    ] {
        output.push_str(&format!(
            "export type {type_name} = typeof {constant}[number];\n"
        ));
    }
    Ok(output)
}

pub fn render_sdk_contract(contract: &Value) -> Result<String, String> {
    let typescript = render_typescript(contract)?;
    let mut output =
        String::from("// Generated from Beaver's extension contract. Do not edit by hand.\n\n");
    for line in typescript.lines().skip(3) {
        if line.starts_with("export const LIMITS") || line.starts_with("export const TIMEOUTS") {
            output.push_str(
                &line
                    .replacen("export const ", "export declare const ", 1)
                    .replace(" = Object.freeze(", ": Readonly<")
                    .replace(" as const);", ">;"),
            );
            output.push('\n');
        } else if line.starts_with("export const ") {
            let (declaration, value) = line
                .trim_end_matches(';')
                .split_once(" = ")
                .ok_or_else(|| "invalid generated SDK declaration".to_string())?;
            let value = value.trim_end_matches(" as const");
            let value = if value.starts_with('[') {
                format!("readonly {value}")
            } else {
                value.to_string()
            };
            output.push_str(&format!(
                "{}: {value};",
                declaration.replacen("export const ", "export declare const ", 1)
            ));
            output.push('\n');
        } else if line.starts_with("export type ") {
            output.push_str(line);
            output.push('\n');
        }
    }
    Ok(output)
}

fn method_names(
    values: &[Value],
    level: Option<&str>,
    kind: Option<&str>,
) -> Result<Vec<Value>, String> {
    values
        .iter()
        .filter(|value| level.is_none_or(|level| value["level"] == level))
        .filter(|value| kind.is_none_or(|kind| value["kind"] == kind))
        .map(|value| {
            value
                .get("name")
                .cloned()
                .ok_or("invalid host method".to_string())
        })
        .collect()
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .get(name)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing extension contract object: {name}"))
}

fn array<'a>(value: &'a Map<String, Value>, name: &str) -> Result<&'a [Value], String> {
    value
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing extension contract array: {name}"))
}

fn array_value<'a>(value: &'a Value, name: &str) -> Result<&'a [Value], String> {
    value
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing extension contract array: {name}"))
}

fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing extension contract string: {name}"))
}

fn json(value: &Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(|_| "cannot render extension contract".to_string())
}
