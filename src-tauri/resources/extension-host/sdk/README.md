# Beaver Extension SDK — API v1

Beaver extensions are trusted local code. They run in a separate Node.js host with the rights and environment of the current user account. The host is not a sandbox.

`access` and `apiLevel` describe compatibility and intended use. They are not process-isolation or security boundaries. Beaver validates registered contributions again in Rust, but extension code remains fully trusted code.

Approval is associated with the extension identity, source, and current bounded file fingerprint. Editing fingerprinted local, Git, or npm content revokes approval, disables the extension, and clears its session permissions before the next load. A Git or npm update managed by Beaver also requests trust again before its next activation. Users remain responsible for auditing every source and update.

## Installation sources

Add a local file or folder directly, install a repository through HTTPS or SSH Git, or install a registry package by npm name. A Git locator can end with `#branch`, `#tag`, or a full commit hash. Keep the Beaver manifest at the repository root or in the published npm package.

Beaver uses its bundled npm with the official HTTPS registry and strict TLS. It installs production dependencies without running npm lifecycle scripts or creating executable links. Repository-level npm configuration is ignored during installation. If your dependency requires an install-time build, prepare it yourself and add the resulting local folder instead.

## Minimal manifest

Create `beaver-extension.json` in the extension folder:

```json
{
  "id": "com.example.hello",
  "name": "Hello",
  "version": "1.0.0",
  "beaverApi": "1",
  "runtime": "node",
  "main": "./index.ts",
  "access": "full",
  "apiLevel": "stable",
  "essential": false
}
```

Set `essential` to `true` only when the plugin's schemas should be loaded first during
progressive discovery. Beaver keeps at most 15 self-declared essential plugins, after
the user's own priority list.

## Minimal extension

```ts
import { defineExtension } from "@beaver/sdk";

export default defineExtension(async (beaver) => {
  beaver.on("session.turn.started", async ({ sessionId }) => {
    // React to a Beaver event.
  });

  beaver.registerTool({
    name: "hello",
    description: "Return a greeting.",
    parameters: {
      type: "object",
      properties: {
        name: { type: "string" }
      },
      required: ["name"],
      additionalProperties: false
    },
    effect: "read-only",
    async execute({ name }) {
      return `Hello ${name}`;
    }
  });
});
```

Beaver namespaces this tool as `com.example.hello.hello`.

## Stable API

- `beaver.info()`
- `beaver.registerTool(definition)`
- `beaver.on(event, handler)`
- `beaver.sessions.list()` and `beaver.sessions.get(id)`
- `beaver.projects.list()`
- `beaver.mcp.listConnectors()` and `beaver.mcp.callTool(...)`
- `beaver.channels.getConfig()`
- `beaver.secrets.getProviderKey(...)`
- `beaver.secrets.getMcpOAuthToken(...)`
- `beaver.secrets.getMcpEnvValue(...)`
- `beaver.secrets.getChannelToken(...)`
- `beaver.call(method, params)` for the versioned low-level bridge

### Standard interface contributions

Declare `ui: { "apiVersion": "1", "mode": "standard" }` in the manifest, then
register declarative contributions without receiving access to Tauri `invoke`:

```js
const remove = beaver.ui.register({
  type: "action",
  id: "hello",
  placement: "app.toolbar.primary",
  order: 0,
  label: { default: "Hello" },
  actionId: "say-hello",
});

const stop = beaver.ui.onAction("say-hello", async (_payload, context) => ({
  type: "notification",
  level: "success",
  message: { default: `Hello (${context.locale})` },
}));
```

Both cleanup functions are idempotent. Beaver namespaces IDs and revalidates every
contribution, action payload, and result in Rust. Invalid UI never removes healthy Tools.
Register contributions during activation: Beaver snapshots the catalog when loading
finishes. Cleanup functions are intended for teardown; later catalog changes become
visible after the next Host reload.

## Advanced API

Set `"apiLevel": "advanced"` to use:

```ts
beaver.unstable.registerReplacement({
  name: "web_search",
  description: "My replacement",
  parameters: { type: "object" },
  effect: "external-read",
  async execute() {
    return "replacement result";
  }
});
```

`beaver.unstable.call(...)` and replacement points may change between Beaver versions.

## Core call errors

Core calls reject with a `BeaverExtensionError`. Use `isBeaverExtensionError(error)` to read its bounded `code`, `reason`, and `retryable` fields. A `core_busy` or `core_request_timeout` error is retryable; an unavailable method is not.

Enabled official plugins share Beaver's official host. Each enabled third-party extension runs in its own managed host process, so disabling or stopping it terminates its code without sharing an identity or failure domain with other third-party extensions.

Secrets are zeroized by Beaver on the Rust side after transfer. Once a secret crosses into JavaScript, immutable strings and the JavaScript garbage collector prevent Beaver from guaranteeing immediate memory erasure.

Safe loading diagnostics (stage, category, source filename, and position when available) appear in **Settings → Extensions → Host and Diagnostics**. Raw extension output is not persisted because it may contain secrets.

The host redirects `console` and ordinary `process.stdout.write` calls to protect the JSON-RPC transport from accidental output. This is not a security boundary: trusted code can still write directly to file descriptor 1 and disrupt its own host.

The extension author and user are responsible for any secret, file, process, or network access performed after activation.

<!-- BEGIN GENERATED EXTENSION CONTRACT -->
### Contract surface

| Category | Values |
|---|---|
| Capabilities | `tools`, `events`, `ui` |
| Core to host | `host.hello`, `host.reset`, `host.load`, `tool.call`, `event.emit`, `ui.action` |
| Events | `session.turn.started` |
| Effects | `read-only`, `local-write`, `external-read`, `external-write`, `process`, `secret`, `unknown` |

### Host to core

| Method | Level | Kind | Rust budget (ms) |
|---|---|---|---:|
| `app.info` | `stable` | `request` | 0 |
| `sessions.list` | `stable` | `request` | 0 |
| `sessions.get` | `stable` | `request` | 0 |
| `projects.list` | `stable` | `request` | 0 |
| `mcp.connectors.list` | `stable` | `request` | 0 |
| `mcp.tool.call` | `stable` | `request` | 25000 |
| `channels.config.get` | `stable` | `request` | 0 |
| `secrets.provider.get` | `stable` | `request` | 0 |
| `secrets.mcp.oauth.get` | `stable` | `request` | 0 |
| `secrets.mcp.env.get` | `stable` | `request` | 0 |
| `secrets.channel.get` | `stable` | `request` | 0 |
| `host.load.stage` | `stable` | `notification` | n/a |

### Limits

| Name | Value |
|---|---:|
| `fingerprintMaxDepth` | 16 |
| `fingerprintMaxFileBytes` | 4194304 |
| `fingerprintMaxFiles` | 2000 |
| `fingerprintMaxTotalBytes` | 33554432 |
| `hostRestartWindowSeconds` | 300 |
| `maxContractCodeChars` | 96 |
| `maxEventsPerExtension` | 64 |
| `maxExtensions` | 132 |
| `maxGitLocatorChars` | 2048 |
| `maxHostProcesses` | 32 |
| `maxHostRestartsPerWindow` | 3 |
| `maxIdentifierChars` | 96 |
| `maxInFlightHandlers` | 64 |
| `maxInFlightRequests` | 64 |
| `maxMessageBytes` | 1048576 |
| `maxNpmSpecChars` | 280 |
| `maxPendingRequests` | 64 |
| `maxPermissionSummaryChars` | 512 |
| `maxProjectResults` | 500 |
| `maxSessionResults` | 500 |
| `maxTools` | 256 |
| `maxToolsPerExtension` | 64 |
| `maxUserExtensions` | 128 |
| `maxWorkingDirectoryChars` | 1024 |
| `minLongLivedAppWorkReserve` | 96 |

### Timeouts

| Name | Value |
|---|---:|
| `coreRequestTimeoutMs` | 30000 |
| `eventHandlerTimeoutMs` | 5000 |
| `hostRequestTimeoutMs` | 60000 |
| `hostStopTimeoutMs` | 5000 |
| `mcpToolTimeoutMs` | 25000 |
| `toolCallTimeoutMs` | 55000 |
| `uiActionTimeoutMs` | 15000 |

### Errors

| Category | Values |
|---|---|
| Protocol reasons | `core_busy`, `core_request_timeout`, `core_transport_failed`, `core_method_unavailable`, `core_request_failed`, `extension_host_busy`, `extension_host_request_failed`, `extension_host_fatal` |
| Backend codes | `extensions_host_unavailable`, `extensions_host_busy`, `extensions_host_timeout`, `extensions_request_too_large`, `extensions_request_invalid`, `extensions_tool_unavailable`, `extensions_tool_arguments_invalid`, `extensions_builtin_catalog_invalid`, `extensions_builtin_catalog_unavailable`, `extensions_builtin_plugin_invalid`, `extensions_builtin_entry_missing`, `extensions_builtin_entry_unavailable`, `extensions_builtin_entry_invalid`, `extensions_install_failed`, `extensions_update_failed`, `extensions_uninstall_failed`, `extensions_source_invalid`, `extensions_package_invalid`, `extensions_git_download_failed`, `extensions_git_timeout`, `extensions_runtime_unavailable`, `extensions_environment_invalid`, `extensions_dependency_install_failed`, `extensions_manifest_invalid`, `extensions_not_beaver_extension`, `extensions_api_incompatible`, `extensions_symlink_unsupported`, `extensions_already_installed`, `extensions_limit_reached`, `extensions_storage_failed`, `extensions_update_identity_changed`, `extensions_update_unavailable`, `extensions_cleanup_failed`, `extensions_operation_failed`, `extensions_fingerprint_changed`, `extensions_fingerprint_failed`, `extensions_stop_unconfirmed`, `extensions_registry_entry_ignored`, `extensions_registry_migration_failed`, `extensions_recovery_marker_invalid`, `extensions_load_interrupted`, `extensions_activation_confirmation_required`, `extensions_not_found`, `extensions_host_incompatible` |
<!-- END GENERATED EXTENSION CONTRACT -->

<!-- BEGIN GENERATED EXTENSION UI CONTRACT -->
### UI contract surface

| Category | Values |
|---|---|
| Modes | `standard`, `advanced` |
| Contribution types | `tab`, `settingsTab`, `action`, `theme` |
| Primitives | `stack`, `row`, `heading`, `text`, `badge`, `separator`, `textField`, `numberField`, `select`, `toggle`, `button` |
| Theme bases | `light`, `dark` |
| Locales | `fr`, `en`, `es`, `de`, `it`, `zh`, `ja` |
| Loading stages | `contract`, `bundle`, `approve`, `import`, `activate`, `mount` |

### UI placements

| Key | Type | Cardinality | Scope |
|---|---|---|---|
| `app.navigation.primary` | `tab` | `multiple` | `global` |
| `settings.navigation.preferences` | `settingsTab` | `multiple` | `global` |
| `settings.navigation.agent` | `settingsTab` | `multiple` | `global` |
| `settings.navigation.models` | `settingsTab` | `multiple` | `global` |
| `settings.navigation.integrations` | `settingsTab` | `multiple` | `global` |
| `settings.navigation.application` | `settingsTab` | `multiple` | `global` |
| `app.toolbar.primary` | `action` | `multiple` | `global` |
| `agent.composer.leading` | `action` | `multiple` | `session` |

### UI limits

| Name | Value |
|---|---:|
| `maxActionPayloadBytes` | 65536 |
| `maxActionResultBytes` | 262144 |
| `maxActionsPerExtension` | 64 |
| `maxAdvancedArtifactBytes` | 4194304 |
| `maxAdvancedArtifactFiles` | 64 |
| `maxAdvancedMountsPerExtension` | 32 |
| `maxContributionsPerExtension` | 32 |
| `maxFieldsPerView` | 32 |
| `maxGlobalStandardContributions` | 512 |
| `maxGlobalUiBytes` | 786432 |
| `maxOccupantsPerPlacement` | 128 |
| `maxOptionsPerField` | 64 |
| `maxTextChars` | 2000 |
| `maxThemeTokens` | 64 |
| `maxThemesPerExtension` | 8 |
| `maxUiBytesPerExtension` | 262144 |
| `maxViewDepth` | 12 |
| `maxViewNodes` | 256 |

### Public UI tokens

`--void`, `--app-frame`, `--shell-opaque`, `--surface`, `--surface-hover`, `--edge`, `--edge-strong`, `--surface-glass`, `--surface-overlay`, `--surface-raised`, `--card-on-glass`, `--field-bg`, `--btn-secondary-bg`, `--ink`, `--ink-muted`, `--ink-faint`, `--ink-on-pulse`, `--ink-on-danger`, `--pulse`, `--pulse-muted`, `--switch-track-off`, `--switch-track-border`, `--switch-track-hover`, `--switch-track-on`, `--switch-knob`, `--signal-error`, `--signal-ok`, `--signal-warning`, `--signal-alert`, `--signal-info`, `--signal-error-bg`, `--signal-ok-bg`, `--signal-warning-bg`, `--signal-alert-bg`, `--signal-info-bg`, `--tooltip-bg`, `--tooltip-border`, `--tooltip-text`, `--chat-composer-bg`, `--chat-composer-border`, `--toast-bg`, `--toast-error-text`, `--toast-ok-text`, `--diff-add-bg`, `--diff-del-bg`, `--diff-new`, `--diff-old`

### UI diagnostics

`ui_contract_invalid`, `ui_contribution_invalid`, `ui_reference_missing`, `ui_reference_incompatible`, `ui_mutation_conflict`, `ui_protected_occupant`, `ui_limit_exceeded`, `ui_bundle_failed`, `ui_node_import_denied`, `ui_artifact_invalid`, `ui_protocol_request_denied`, `ui_protocol_hash_mismatch`, `ui_protocol_file_missing`, `ui_protocol_file_too_large`, `ui_advanced_approval_required`, `ui_activation_failed`, `ui_mount_failed`, `ui_theme_unavailable`
<!-- END GENERATED EXTENSION UI CONTRACT -->
