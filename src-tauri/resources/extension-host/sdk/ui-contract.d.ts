// Generated from src-tauri/resources/extension-ui/contract.json.
// Do not edit by hand.

export declare const EXTENSION_UI_API_VERSION: "1";
export declare const UI_MODES: readonly ["standard","advanced"];
export declare const UI_CONTRIBUTION_TYPES: readonly ["tab","settingsTab","action","theme"];
export declare const UI_PRIMITIVES: readonly ["stack","row","heading","text","badge","separator","textField","numberField","select","toggle","button"];
export declare const UI_THEME_BASES: readonly ["light","dark"];
export declare const UI_LOCALES: readonly ["fr","en","es","de","it","zh","ja"];
export declare const UI_PLACEMENT_OPERATIONS: readonly ["before","after","replace","move","remove"];
export declare const UI_PLACEMENTS: readonly [{"cardinality":"multiple","contributionType":"tab","key":"app.navigation.primary","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.preferences","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.agent","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.models","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.integrations","scope":"global"},{"cardinality":"multiple","contributionType":"settingsTab","key":"settings.navigation.application","scope":"global"},{"cardinality":"multiple","contributionType":"action","key":"app.toolbar.primary","scope":"global"},{"cardinality":"multiple","contributionType":"action","key":"agent.composer.leading","scope":"session","thirdPartyChatAllowed":false}];
export declare const UI_PROTECTED_OCCUPANTS: readonly [{"occupant":"beaver.settings","operations":["remove","replace"],"placement":"app.navigation.primary"},{"occupant":"beaver.extensions","operations":["remove","replace"],"placement":"settings.navigation.integrations"}];
export declare const UI_ICONS: readonly ["activity","archive","bell","book-open","brain","check","chevron-down","circle","gear","house","info","link","moon","plus","puzzle-piece","sparkle","sun","terminal","warning","x"];
export declare const UI_THEME_TOKENS: readonly ["--void","--app-frame","--shell-opaque","--surface","--surface-hover","--edge","--edge-strong","--surface-glass","--surface-overlay","--surface-raised","--card-on-glass","--field-bg","--btn-secondary-bg","--ink","--ink-muted","--ink-faint","--ink-on-pulse","--ink-on-danger","--pulse","--pulse-muted","--switch-track-off","--switch-track-border","--switch-track-hover","--switch-track-on","--switch-knob","--signal-error","--signal-ok","--signal-warning","--signal-alert","--signal-info","--signal-error-bg","--signal-ok-bg","--signal-warning-bg","--signal-alert-bg","--signal-info-bg","--tooltip-bg","--tooltip-border","--tooltip-text","--chat-composer-bg","--chat-composer-border","--toast-bg","--toast-error-text","--toast-ok-text","--diff-add-bg","--diff-del-bg","--diff-new","--diff-old"];
export declare const UI_LOADING_STAGES: readonly ["contract","bundle","approve","import","activate","mount"];
export declare const UI_DIAGNOSTIC_CODES: readonly ["ui_contract_invalid","ui_contribution_invalid","ui_reference_missing","ui_reference_incompatible","ui_mutation_conflict","ui_protected_occupant","ui_limit_exceeded","ui_bundle_failed","ui_node_import_denied","ui_artifact_invalid","ui_protocol_request_denied","ui_protocol_hash_mismatch","ui_protocol_file_missing","ui_protocol_file_too_large","ui_advanced_approval_required","ui_activation_failed","ui_mount_failed","ui_theme_unavailable"];
export declare const UI_LIMITS: Readonly<{"maxActionPayloadBytes":65536,"maxActionResultBytes":262144,"maxActionsPerExtension":64,"maxAdvancedActivationMs":15000,"maxAdvancedArtifactBytes":4194304,"maxAdvancedArtifactFiles":64,"maxAdvancedMountsPerExtension":32,"maxContributionsPerExtension":32,"maxFieldsPerView":32,"maxGlobalStandardContributions":512,"maxGlobalUiBytes":786432,"maxInFlightActionsPerExtension":8,"maxOccupantsPerPlacement":128,"maxOptionsPerField":64,"maxTextChars":2000,"maxThemeTokens":64,"maxThemesPerExtension":8,"maxUiBytesPerExtension":262144,"maxViewDepth":12,"maxViewNodes":256}>;
export declare const UI_VALIDATION: Readonly<{"maxNumericLimit":4194304,"maxOrder":1000,"minOrder":-1000,"themeValuePattern":"^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$"}>;

export type ExtensionUiMode = typeof UI_MODES[number];
export type ExtensionUiContributionType = typeof UI_CONTRIBUTION_TYPES[number];
export type ExtensionUiPrimitive = typeof UI_PRIMITIVES[number];
export type ExtensionUiPlacementKey = typeof UI_PLACEMENTS[number]["key"];
export type ExtensionUiIcon = typeof UI_ICONS[number];
export type ExtensionUiThemeToken = typeof UI_THEME_TOKENS[number];
export type ExtensionUiLoadingStage = typeof UI_LOADING_STAGES[number];
export type ExtensionUiDiagnosticCode = typeof UI_DIAGNOSTIC_CODES[number];
