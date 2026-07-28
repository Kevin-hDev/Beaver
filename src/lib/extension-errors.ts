export const EXTENSION_BACKEND_ERROR_CODES = Object.freeze([
  "extensions_builtin_catalog_invalid",
  "extensions_host_unavailable",
  "extensions_builtin_plugin_invalid",
  "extensions_builtin_catalog_unavailable",
  "extensions_builtin_entry_missing",
  "extensions_builtin_entry_unavailable",
  "extensions_builtin_entry_invalid",
  "extensions_host_busy",
  "extensions_host_timeout",
  "extensions_request_too_large",
  "extensions_request_invalid",
  "extensions_tool_unavailable",
  "extensions_tool_arguments_invalid",
  "extensions_install_failed",
  "extensions_update_failed",
  "extensions_uninstall_failed",
  "extensions_source_invalid",
  "extensions_package_invalid",
  "extensions_git_download_failed",
  "extensions_git_timeout",
  "extensions_runtime_unavailable",
  "extensions_dependency_install_failed",
  "extensions_manifest_invalid",
  "extensions_already_installed",
  "extensions_limit_reached",
  "extensions_storage_failed",
  "extensions_update_identity_changed",
  "extensions_update_unavailable",
  "extensions_cleanup_failed",
]);
const BACKEND_CODES = new Set(EXTENSION_BACKEND_ERROR_CODES);

export function extensionErrorKey(error: unknown, fallback: string) {
  const code = typeof error === "string"
    ? error
    : error instanceof Error
      ? error.message
      : "";
  return BACKEND_CODES.has(code)
    ? `extensions.errors.codes.${code}`
    : fallback;
}
