// Generated from src-tauri/resources/extension-host/contract.json.
// Do not edit by hand.

export const EXTENSION_API_VERSION = "1" as const;
export const EXTENSION_CAPABILITIES = ["tools","events"] as const;
export const CORE_TO_HOST_METHODS = ["host.hello","host.reset","host.load","tool.call","event.emit"] as const;
export const STABLE_HOST_TO_CORE_REQUEST_METHODS = ["app.info","sessions.list","sessions.get","projects.list","mcp.connectors.list","mcp.tool.call","channels.config.get","secrets.provider.get","secrets.mcp.oauth.get","secrets.mcp.env.get","secrets.channel.get"] as const;
export const ADVANCED_HOST_TO_CORE_REQUEST_METHODS = [] as const;
export const HOST_TO_CORE_NOTIFICATION_METHODS = ["host.load.stage"] as const;
export const EXTENSION_EVENTS = ["session.turn.started"] as const;
export const EXTENSION_HOST_STATES = ["stopped","starting","running","error"] as const;
export const HOST_LOAD_STAGES = ["import","activate","register"] as const;
export const EXTENSION_EFFECT_CLASSES = ["read-only","local-write","external-read","external-write","process","secret","unknown"] as const;
export const PROTOCOL_ERROR_REASONS = ["core_busy","core_request_timeout","core_transport_failed","core_method_unavailable","core_request_failed","extension_host_busy","extension_host_request_failed","extension_host_fatal"] as const;
export const EXTENSION_BACKEND_ERROR_CODES = ["extensions_host_unavailable","extensions_host_busy","extensions_host_timeout","extensions_request_too_large","extensions_request_invalid","extensions_tool_unavailable","extensions_tool_arguments_invalid","extensions_builtin_catalog_invalid","extensions_builtin_catalog_unavailable","extensions_builtin_plugin_invalid","extensions_builtin_entry_missing","extensions_builtin_entry_unavailable","extensions_builtin_entry_invalid","extensions_install_failed","extensions_update_failed","extensions_uninstall_failed","extensions_source_invalid","extensions_package_invalid","extensions_git_download_failed","extensions_git_timeout","extensions_runtime_unavailable","extensions_environment_invalid","extensions_dependency_install_failed","extensions_manifest_invalid","extensions_not_beaver_extension","extensions_api_incompatible","extensions_symlink_unsupported","extensions_already_installed","extensions_limit_reached","extensions_storage_failed","extensions_update_identity_changed","extensions_update_unavailable","extensions_cleanup_failed","extensions_operation_failed","extensions_fingerprint_changed","extensions_fingerprint_failed","extensions_stop_unconfirmed","extensions_registry_entry_ignored","extensions_registry_migration_failed","extensions_recovery_marker_invalid","extensions_load_interrupted","extensions_activation_confirmation_required","extensions_not_found","extensions_host_incompatible"] as const;
export const HOST_DIAGNOSTIC_CODES = ["module_not_found","syntax_error","activation_failed","registration_failed","import_failed"] as const;
export const RUNTIME_DIAGNOSTIC_CODES = ["entry_unavailable","load_failed","host_missing_response","advanced_required"] as const;
export const LIMITS = Object.freeze({"fingerprintMaxDepth":16,"fingerprintMaxFileBytes":4194304,"fingerprintMaxFiles":2000,"fingerprintMaxTotalBytes":33554432,"hostRestartWindowSeconds":300,"maxContractCodeChars":96,"maxEventsPerExtension":64,"maxExtensions":132,"maxGitLocatorChars":2048,"maxHostProcesses":32,"maxHostRestartsPerWindow":3,"maxIdentifierChars":96,"maxInFlightHandlers":64,"maxInFlightRequests":64,"maxMessageBytes":1048576,"maxNpmSpecChars":280,"maxPendingRequests":64,"maxPermissionSummaryChars":512,"maxProjectResults":500,"maxSessionResults":500,"maxTools":256,"maxToolsPerExtension":64,"maxUserExtensions":128,"maxWorkingDirectoryChars":1024,"minLongLivedAppWorkReserve":96} as const);
export const TIMEOUTS = Object.freeze({"coreRequestTimeoutMs":30000,"eventHandlerTimeoutMs":5000,"hostRequestTimeoutMs":60000,"hostStopTimeoutMs":5000,"mcpToolTimeoutMs":25000,"toolCallTimeoutMs":55000} as const);

export type ExtensionCapability = typeof EXTENSION_CAPABILITIES[number];
export type CoreToHostMethod = typeof CORE_TO_HOST_METHODS[number];
export type StableHostToCoreRequestMethod = typeof STABLE_HOST_TO_CORE_REQUEST_METHODS[number];
export type AdvancedHostToCoreRequestMethod = typeof ADVANCED_HOST_TO_CORE_REQUEST_METHODS[number];
export type HostToCoreNotificationMethod = typeof HOST_TO_CORE_NOTIFICATION_METHODS[number];
export type ExtensionEvent = typeof EXTENSION_EVENTS[number];
export type ExtensionHostState = typeof EXTENSION_HOST_STATES[number];
export type HostLoadStage = typeof HOST_LOAD_STAGES[number];
export type ExtensionEffectClass = typeof EXTENSION_EFFECT_CLASSES[number];
export type ExtensionProtocolErrorReason = typeof PROTOCOL_ERROR_REASONS[number];
export type ExtensionBackendErrorCode = typeof EXTENSION_BACKEND_ERROR_CODES[number];
export type HostDiagnosticCode = typeof HOST_DIAGNOSTIC_CODES[number];
export type RuntimeDiagnosticCode = typeof RUNTIME_DIAGNOSTIC_CODES[number];
