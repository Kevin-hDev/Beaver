use super::AutomationError;

pub(super) fn typescript_contract() -> String {
    let codes = AutomationError::ALL
        .iter()
        .map(|error| format!("\"{}\"", error.code()))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "// Generated from services/automations/types.rs. Do not edit manually.\n\
export const AUTOMATION_ERROR_CODES = [{codes}] as const;\n\
export type AutomationErrorCode = typeof AUTOMATION_ERROR_CODES[number];\n"
    )
}

#[test]
fn checked_in_automation_contract_matches_rust() {
    let checked_in = include_str!("../../../../src/types/automation-contract.generated.ts")
        .replace("\r\n", "\n");
    assert_eq!(checked_in, typescript_contract());
    let mut codes = AutomationError::ALL
        .iter()
        .map(|error| error.code())
        .collect::<Vec<_>>();
    let before = codes.len();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), before);
}

#[test]
#[ignore = "developer command that refreshes the automation TypeScript contract"]
fn export_typescript_automation_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/types/automation-contract.generated.ts");
    std::fs::write(path, typescript_contract()).unwrap();
}
