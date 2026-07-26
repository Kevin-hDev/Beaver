const contract = (name, file, snippets) => ({ name, file, snippets });

export const MAC_CEF_HELPERS = Object.freeze([
  Object.freeze(["principal", "Beaver Helper.app", "com.clgo.dash.helper"]),
  Object.freeze(["GPU", "Beaver Helper (GPU).app", "com.clgo.dash.helper.gpu"]),
  Object.freeze([
    "renderer",
    "Beaver Helper (Renderer).app",
    "com.clgo.dash.helper.renderer",
  ]),
  Object.freeze(["plugin", "Beaver Helper (Plugin).app", "com.clgo.dash.helper.plugin"]),
  Object.freeze(["alerts", "Beaver Helper (Alerts).app", "com.clgo.dash.helper.alerts"]),
]);

const cefHelpers = MAC_CEF_HELPERS.map(([name, directory, identifier]) =>
  contract(
    `bundle ID CEF ${name}`,
    `src-tauri/resources/cef/macos/helpers/${directory}/Contents/Info.plist`,
    [`<string>${identifier}</string>`],
  ),
);

export const PLATFORM_COMPATIBILITY_CONTRACTS = Object.freeze([
  contract("bundle ID Tauri", "src-tauri/tauri.conf.json", [
    '"identifier": "com.clgo.dash"',
  ]),
  contract("bundle ID CEF principal", "src-tauri/resources/cef/macos/dev-app/Info.plist", [
    "<string>com.clgo.dash</string>",
  ]),
  ...cefHelpers,
  contract("migration du paquet Debian", "src-tauri/tauri.conf.json", [
    '"provides": ["cl-go"]',
    '"conflicts": ["cl-go"]',
    '"replaces": ["cl-go"]',
    '"installerHooks": "windows/nsis-hooks.nsh"',
  ]),
  contract("validation du paquet Debian téléchargé", "install.sh", [
    'Provides 2>/dev/null)" = "cl-go"',
    'Conflicts 2>/dev/null)" = "cl-go"',
    'Replaces 2>/dev/null)" = "cl-go"',
  ]),
  contract("migration de l’installateur Windows", "src-tauri/windows/nsis-hooks.nsh", [
    '!define BEAVER_OLD_UNINSTALL "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CL-GO"',
    '!define BEAVER_OLD_PRODUCT "Software\\clgo\\CL-GO"',
    '!define BEAVER_MAIN_BINARY "cl-go-dash.exe"',
    'DeleteRegKey SHCTX "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CL-GO"',
  ]),
]);
