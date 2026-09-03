// Generated from src-tauri/resources/extension-ui/contract.json.
// Do not edit by hand.

export const EXTENSION_UI_API_VERSION = "1" as const;
export const UI_MODES = ["standard","advanced"] as const;
export const UI_CONTRIBUTION_TYPES = ["tab","settingsTab","action","theme"] as const;
export const UI_PRIMITIVES = ["stack","row","heading","text","badge","separator","textField","numberField","select","toggle","button"] as const;
export const UI_THEME_BASES = ["light","dark"] as const;
export const UI_LOCALES = ["fr","en","es","de","it","zh","ja"] as const;
export const UI_PLACEMENT_OPERATIONS = ["before","after","replace","move","remove"] as const;
export const UI_PLACEMENTS = [{"cardinality":"multiple","contributionType":"tab","key":"app.navigation.primary","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.preferences","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.agent","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.models","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.integrations","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.application","scope":"global"},{"cardinality":"multiple","contributionType":"action","key":"app.toolbar.primary","scope":"global"},{"cardinality":"multiple","contributionType":"action","key":"agent.composer.leading","scope":"session","thirdPartyChatAllowed":false}] as const;
export const UI_PROTECTED_OCCUPANTS = [{"occupant":"beaver.settings","operations":["remove","replace"],"placement":"app.navigation.primary"},{"occupant":"beaver.extensions","operations":["remove","replace"],"placement":"settings.navigation.integrations"}] as const;
export const UI_ICONS = ["activity","archive","bell","book-open","brain","check","chevron-down","circle","gear","house","info","link","moon","plus","puzzle-piece","sparkle","sun","terminal","warning","x"] as const;
export const UI_THEME_TOKENS = ["--app-frame","--shell","--surface","--surface-hover","--surface-overlay","--surface-raised","--card-on-glass","--field-bg","--select-bg","--select-text","--edge","--edge-strong","--border-subtle","--ink","--ink-muted","--ink-faint","--ink-primary","--text-primary","--text-secondary","--text-tertiary","--pulse","--pulse-hover","--pulse-muted","--danger","--signal-alert","--signal-error","--signal-error-bg","--signal-info","--signal-info-bg","--signal-ok","--signal-ok-bg","--signal-warning","--signal-warning-bg","--chat-composer-bg","--chat-composer-border","--toast-bg","--toast-error-text","--toast-ok-text","--diff-add-bg","--diff-add-gutter","--diff-del-bg","--diff-del-gutter","--diff-new","--diff-old"] as const;
export const UI_LOADING_STAGES = ["contract","bundle","approve","import","activate","mount"] as const;
export const UI_DIAGNOSTIC_CODES = ["ui_contract_invalid","ui_contribution_invalid","ui_reference_missing","ui_reference_incompatible","ui_mutation_conflict","ui_protected_occupant","ui_limit_exceeded","ui_bundle_failed","ui_node_import_denied","ui_artifact_invalid","ui_protocol_request_denied","ui_protocol_hash_mismatch","ui_protocol_file_missing","ui_protocol_file_too_large","ui_advanced_approval_required","ui_activation_failed","ui_mount_failed","ui_theme_unavailable"] as const;
export const UI_LIMITS = Object.freeze({"maxActionPayloadBytes":65536,"maxActionResultBytes":262144,"maxActionsPerExtension":64,"maxAdvancedArtifactBytes":4194304,"maxAdvancedArtifactFiles":64,"maxAdvancedMountsPerExtension":32,"maxContributionsPerExtension":32,"maxFieldsPerView":32,"maxGlobalStandardContributions":512,"maxGlobalUiBytes":786432,"maxOccupantsPerPlacement":128,"maxOptionsPerField":64,"maxTextChars":2000,"maxThemeTokens":64,"maxThemesPerExtension":8,"maxUiBytesPerExtension":262144,"maxViewDepth":12,"maxViewNodes":256} as const);
export const UI_VALIDATION = Object.freeze({"maxNumericLimit":4194304,"maxOrder":1000,"minOrder":-1000,"themeValuePattern":"^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$"} as const);

export type ExtensionUiMode = typeof UI_MODES[number];
export type ExtensionUiContributionType = typeof UI_CONTRIBUTION_TYPES[number];
export type ExtensionUiPrimitive = typeof UI_PRIMITIVES[number];
export type ExtensionUiPlacementKey = typeof UI_PLACEMENTS[number]["key"];
export type ExtensionUiIcon = typeof UI_ICONS[number];
export type ExtensionUiThemeToken = typeof UI_THEME_TOKENS[number];
export type ExtensionUiLoadingStage = typeof UI_LOADING_STAGES[number];
export type ExtensionUiDiagnosticCode = typeof UI_DIAGNOSTIC_CODES[number];
