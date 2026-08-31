import { PLATFORM_COMPATIBILITY_CONTRACTS } from "./brand-boundaries-platform-contracts.mjs";
import { UPDATE_COMPATIBILITY_CONTRACTS } from "./brand-boundaries-update-contracts.mjs";

const contract = (name, file, snippets) => ({ name, file, snippets });

export const EXPECTED_INTERNAL_REFERENCE_COUNTS = Object.freeze({
  "CL-GO-DASH": 3,
  "CL-GO": 38,
  CLGO: 32,
  "cl-go-dash": 253,
  clgo: 114,
  "cl-go": 72,
  cl_go_dash: 46,
});

export const COMPATIBILITY_CONTRACTS = Object.freeze([
  ...PLATFORM_COMPATIBILITY_CONTRACTS,
  contract("dossier de données", "src-tauri/src/services/paths.rs", [
    '.join(".local/share/cl-go-dash")',
  ]),
  contract("migration des anciens dossiers", "src-tauri/src/storage_migration.rs", [
    'home.join(".local/share/cl-go")',
    'new.join(".migrated-from-cl-go")',
    'd.join("cl-go-dash")',
  ]),
  contract("coffre et keyring", "src-tauri/src/services/vault.rs", [
    'const KEYRING_SERVICE: &str = "cl-go-dash";',
    'const MASTER_KEY_USER: &str = "master-key";',
    'data_dir().join("secrets.enc")',
  ]),
  contract("domaine des pièces jointes", "src-tauri/src/services/attachment_access.rs", [
    'b"cl-go-dash:attachment-access:v1"',
  ]),
  contract("package npm", "package.json", ['"name": "cl-go-dash"']),
  contract(
    "validation des artefacts Beaver",
    "scripts/release/brand-artifact-common.mjs",
    ['packageJson.name !== "cl-go-dash"', 'config.identifier !== "com.clgo.dash"'],
  ),
  contract(
    "validation des bundles macOS Beaver",
    "scripts/release/check-brand-artifacts.mjs",
    ['"com.clgo.dash"', '"cl-go-dash"', "Contents/MacOS/${executable}"],
  ),
  contract("crates et exécutables Rust", "src-tauri/Cargo.toml", [
    'name = "cl-go-dash"',
    'name = "cl_go_dash_lib"',
    'name = "cl-go-dash-helper"',
    'name = "cl-go-dash-updater"',
  ]),
  ...UPDATE_COMPATIBILITY_CONTRACTS,
  contract("entrée Rust", "src-tauri/src/main.rs", ["cl_go_dash_lib::"]),
  contract("binaire Windows", "src-tauri/scripts/prepare-cef-windows.ps1", [
    '"cl_go_dash_lib.dll"',
    '"cl-go-dash.exe"',
  ]),
  contract("variables CEF", "src-tauri/scripts/prepare-cef.sh", [
    "CLGO_CEF_DEV_PREP",
    "CLGO_CEF_ALLOW_ADHOC_SIGNING",
    "cl-go-dash-helper",
  ]),
  contract("langue persistante", "src/i18n/index.ts", ["clgo-language"]),
  contract("thème persistant", "src/hooks/use-theme.ts", ["clgo-theme"]),
  contract("réglages persistants", "src/hooks/use-settings.ts", [
    "clgo-font-size",
    "clgo-font-family",
    "clgo-code-theme",
  ]),
  contract("aperçus persistants", "src/hooks/file-preview-storage.ts", [
    "clgo-file-preview-tabs:",
    "clgo-file-preview-panel:",
  ]),
  contract("arbre de fichiers persistant", "src/hooks/use-file-tree.ts", [
    "clgo-file-tree-width:",
  ]),
  contract("événements frontend", "src/hooks/agent-session-events.ts", [
    "clgo-agent-sessions-changed",
  ]),
  contract("réglages avancés", "src/hooks/use-setting-value.ts", [
    "clgo-advanced-settings-changed",
  ]),
  contract(
    "occlusion du navigateur",
    "src/components/internal-browser/browser-native-occlusion.ts",
    ["clgo-browser-native-occlusion-v1"],
  ),
  contract(
    "conversations repliées",
    "src/components/agent-local/use-conversation-collapse-state.ts",
    ["clgo-conversation-collapse-v1"],
  ),
  contract("aperçu des liens", "src/components/settings/general-settings.tsx", [
    "clgo-link-preview",
  ]),
  contract("événement OAuth", "src/hooks/oauth-models.ts", [
    "cl-go:oauth-models-updated",
  ]),
  contract("argument autostart", "src-tauri/src/app_events.rs", [
    '"--clgo-autostart"',
  ]),
  contract(
    "migration de l’entrée autostart historique",
    "src-tauri/src/services/autostart_migration.rs",
    ['const LEGACY_ENTRY_NAME: &str = "CL-GO";'],
  ),
  contract("règles projet", "src-tauri/src/services/agent_local/agent_md.rs", [
    '.join(".cl-go")',
  ]),
  contract(
    "branches de sous-agents",
    "src-tauri/src/services/agent_local/subagent_directory_change.rs",
    ['"cl-go/directory"', '"cl-go/subagent/"'],
  ),
  contract("worktrees de sous-agents", "src-tauri/src/services/agent_local/subagent_worktree.rs", [
    '"cl-go/subagent/{execution_id}"',
  ]),
  contract(
    "transactions de sous-agents",
    "src-tauri/src/services/agent_local/subagent_directory_transaction.rs",
    ['".cl-go-transaction-{transaction_id}"', '".cl-go-{}.tmp"'],
  ),
  contract("mascotte Rust", "src-tauri/src/models/mascot.rs", ['"cl-go-beaver"']),
  contract("mascotte frontend", "src/types/mascot.ts", ['"cl-go-beaver"']),
  contract("manifest de mascotte", "src/assets/mascot/cl-go-beaver/manifest.json", [
    '"id": "cl-go-beaver"',
  ]),
  contract("clés OAuth LLM", "src-tauri/src/services/api_keys_credential_scope_wire.rs", [
    '"_llm_oauth_xai"',
    '"_llm_oauth_kimi"',
  ]),
  contract("contrat OAuth xAI", "src-tauri/src/services/llm_oauth/xai.rs", [
    '"b1a00492-073a-47ea-816f-4c329264a828"',
    '"http://127.0.0.1:56121/callback"',
    '"cl-go-dash"',
  ]),
  contract("contrat OAuth Kimi", "src-tauri/src/services/llm_oauth/kimi.rs", [
    '"17e5f671-d194-4dfb-9706-5516cb48c098"',
  ]),
  contract("identité Kimi", "src-tauri/src/services/llm_oauth/headers.rs", [
    '"kimi_code_cli"',
    '"oauth-providers/kimi-device-id"',
  ]),
  contract("identifiants MCP", "src-tauri/src/services/mcp_oauth/static_credentials.rs", [
    '"_oauth_google_client_id"',
    '"_oauth_google_client_secret"',
    '"_oauth_github_client_id"',
    '"_oauth_github_client_secret"',
    '"CLGO_GOOGLE_CLIENT_ID"',
    '"CLGO_GOOGLE_CLIENT_SECRET"',
    '"CLGO_GITHUB_CLIENT_ID"',
    '"CLGO_GITHUB_CLIENT_SECRET"',
  ]),
  contract("serveur Forecast", "src-tauri/resources/forecast-sidecar/server.py", [
    '"x-clgo-forecast-token"',
    '"CLGO_FORECAST_TOKEN"',
    '"CLGO_FORECAST_DEVICE"',
  ]),
  contract("lancement Forecast", "src-tauri/src/services/forecast/sidecar_spawn.rs", [
    '"CLGO_FORECAST_TOKEN"',
  ]),
  contract("réglages Forecast", "src-tauri/src/services/forecast/sidecar_settings.rs", [
    '"CLGO_FORECAST_DEVICE"',
    '"CLGO_FORECAST_KEEP_ALIVE"',
  ]),
  contract("client Forecast", "src-tauri/src/services/forecast/client_chronos.rs", [
    '"X-CLGO-Forecast-Token"',
  ]),
  contract("sonde Forecast", "src-tauri/src/services/forecast/sidecar_http.rs", [
    "X-CLGO-Forecast-Token",
  ]),
  contract("empreinte Forecast", "src-tauri/src/services/forecast/data_fingerprint.rs", [
    'b"cl-go-forecast-input-v1"',
  ]),
  // Le décompte des références internes est global : il ne casse que sur la tête
  // complète, jamais sur un diff isolé. Tant que ci.yml ne l'exécute pas, une
  // régression reste invisible jusqu'à la publication.
  contract("exécution du contrat en intégration continue", ".github/workflows/ci.yml", [
    "npm run test:brand-boundaries",
  ]),
]);
