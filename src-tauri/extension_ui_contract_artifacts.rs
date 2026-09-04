use serde_json::Value;

const SOURCE: &str = "src-tauri/resources/extension-ui/contract.json";

pub fn render_typescript(contract: &Value) -> Result<String, String> {
    let mut output = header("//", SOURCE);
    render_constants(&mut output, contract, true)?;
    output.push_str("\nexport type ExtensionUiMode = typeof UI_MODES[number];\n");
    output.push_str(
        "export type ExtensionUiContributionType = typeof UI_CONTRIBUTION_TYPES[number];\n",
    );
    output.push_str("export type ExtensionUiPrimitive = typeof UI_PRIMITIVES[number];\n");
    output
        .push_str("export type ExtensionUiPlacementKey = typeof UI_PLACEMENTS[number][\"key\"];\n");
    output.push_str("export type ExtensionUiIcon = typeof UI_ICONS[number];\n");
    output.push_str("export type ExtensionUiThemeToken = typeof UI_THEME_TOKENS[number];\n");
    output.push_str("export type ExtensionUiLoadingStage = typeof UI_LOADING_STAGES[number];\n");
    output
        .push_str("export type ExtensionUiDiagnosticCode = typeof UI_DIAGNOSTIC_CODES[number];\n");
    Ok(output)
}

pub fn render_sdk(contract: &Value) -> Result<String, String> {
    let mut output = header("//", SOURCE);
    output.push_str(&format!(
        "export declare const EXTENSION_UI_API_VERSION: {};\n",
        json(&contract["apiVersion"])?
    ));
    for (name, key) in constant_arrays() {
        output.push_str(&format!(
            "export declare const {name}: readonly {};\n",
            json(&contract[key])?
        ));
    }
    for (name, key) in constant_objects() {
        output.push_str(&format!(
            "export declare const {name}: Readonly<{}>;\n",
            json(&contract[key])?
        ));
    }
    output.push('\n');
    append_types(&mut output);
    Ok(output)
}

pub fn render_node(contract: &Value) -> Result<String, String> {
    let mut output = header("//", SOURCE);
    render_constants(&mut output, contract, false)?;
    Ok(output)
}

fn render_constants(output: &mut String, contract: &Value, typescript: bool) -> Result<(), String> {
    output.push_str(&format!(
        "export const EXTENSION_UI_API_VERSION = {}{};\n",
        json(&contract["apiVersion"])?,
        suffix(typescript)
    ));
    for (name, key) in constant_arrays() {
        output.push_str(&format!(
            "export const {name} = {}{};\n",
            json(&contract[key])?,
            suffix(typescript)
        ));
    }
    for (name, key) in constant_objects() {
        let object = contract[key]
            .as_object()
            .ok_or_else(|| format!("missing extension UI {key}"))?;
        output.push_str(&format!(
            "export const {name} = Object.freeze({}{});\n",
            json(&Value::Object(object.clone()))?,
            suffix(typescript)
        ));
    }
    Ok(())
}

fn constant_arrays() -> [(&'static str, &'static str); 12] {
    [
        ("UI_MODES", "modes"),
        ("UI_CONTRIBUTION_TYPES", "contributionTypes"),
        ("UI_PRIMITIVES", "primitives"),
        ("UI_THEME_BASES", "themeBases"),
        ("UI_LOCALES", "locales"),
        ("UI_PLACEMENT_OPERATIONS", "placementOperations"),
        ("UI_PLACEMENTS", "placements"),
        ("UI_PROTECTED_OCCUPANTS", "protectedOccupants"),
        ("UI_ICONS", "icons"),
        ("UI_THEME_TOKENS", "themeTokens"),
        ("UI_LOADING_STAGES", "loadingStages"),
        ("UI_DIAGNOSTIC_CODES", "diagnosticCodes"),
    ]
}

fn constant_objects() -> [(&'static str, &'static str); 2] {
    [("UI_LIMITS", "limits"), ("UI_VALIDATION", "validation")]
}

fn append_types(output: &mut String) {
    output.push_str("export type ExtensionUiMode = typeof UI_MODES[number];\n");
    output.push_str(
        "export type ExtensionUiContributionType = typeof UI_CONTRIBUTION_TYPES[number];\n",
    );
    output.push_str("export type ExtensionUiPrimitive = typeof UI_PRIMITIVES[number];\n");
    output
        .push_str("export type ExtensionUiPlacementKey = typeof UI_PLACEMENTS[number][\"key\"];\n");
    output.push_str("export type ExtensionUiIcon = typeof UI_ICONS[number];\n");
    output.push_str("export type ExtensionUiThemeToken = typeof UI_THEME_TOKENS[number];\n");
    output.push_str("export type ExtensionUiLoadingStage = typeof UI_LOADING_STAGES[number];\n");
    output
        .push_str("export type ExtensionUiDiagnosticCode = typeof UI_DIAGNOSTIC_CODES[number];\n");
}

fn suffix(typescript: bool) -> &'static str {
    if typescript {
        " as const"
    } else {
        ""
    }
}

fn header(comment: &str, source: &str) -> String {
    format!("{comment} Generated from {source}.\n{comment} Do not edit by hand.\n\n")
}

fn json(value: &Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(|_| "cannot render extension UI contract".to_string())
}
