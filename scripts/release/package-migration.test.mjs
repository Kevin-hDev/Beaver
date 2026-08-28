import assert from "node:assert/strict";
import { readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { load as loadYaml } from "js-yaml";

const ROOT = fileURLToPath(new URL("../../", import.meta.url));
const MAX_SOURCE_BYTES = 64 * 1024;

function readBounded(relativePath) {
  const path = resolve(ROOT, relativePath);
  const stat = statSync(path);
  assert.equal(stat.isFile(), true);
  assert.ok(stat.size > 0 && stat.size <= MAX_SOURCE_BYTES);
  return readFileSync(path, "utf8");
}

test("le paquet Debian Beaver remplace uniquement le paquet historique", () => {
  const config = JSON.parse(readBounded("src-tauri/tauri.conf.json"));

  assert.equal(config.productName, "Beaver");
  assert.equal(config.identifier, "com.clgo.dash");
  assert.deepEqual(config.bundle.linux.deb, {
    provides: ["cl-go"],
    conflicts: ["cl-go"],
    replaces: ["cl-go"],
  });
});

test("les paquets livrent la licence sans dialogue bloquant dans le DMG", () => {
  const config = JSON.parse(readBounded("src-tauri/tauri.conf.json"));
  const windows = JSON.parse(readBounded("src-tauri/tauri.windows.conf.json"));

  assert.equal(config.bundle.licenseFile, undefined);
  assert.ok(config.bundle.resources.includes("../LICENSE"));
  assert.equal(windows.bundle.resources["../LICENSE"], "LICENSE.txt");
});

test("le bundle Windows utilise le hook de migration dédié", () => {
  const config = JSON.parse(readBounded("src-tauri/tauri.conf.json"));

  assert.equal(config.bundle.windows.nsis.installMode, "currentUser");
  assert.equal(
    config.bundle.windows.nsis.installerHooks,
    "windows/nsis-hooks.nsh",
  );
});

test("le bundle Windows conserve le helper au chemin attendu par l'application", () => {
  const config = JSON.parse(readBounded("src-tauri/tauri.windows.conf.json"));
  const helper = "target/updater-helper/cl-go-dash-updater.exe";

  assert.equal(config.bundle.resources[helper], helper);
});

test("le hook Windows valide avant de nettoyer les anciennes métadonnées", () => {
  const hook = readBounded("src-tauri/windows/nsis-hooks.nsh");
  const oldUninstall =
    "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\CL-GO";
  const newUninstall =
    "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Beaver";
  const oldProduct = "Software\\clgo\\CL-GO";
  const newProduct = "Software\\clgo\\Beaver";

  for (const value of [
    oldUninstall,
    newUninstall,
    oldProduct,
    newProduct,
    "cl-go-dash.exe",
    "NSIS_HOOK_PREINSTALL",
    "NSIS_HOOK_POSTINSTALL",
    "SetOutPath $INSTDIR",
  ]) {
    assert.ok(hook.includes(value), `contrat absent: ${value}`);
  }

  assert.ok(hook.indexOf(newUninstall) < hook.lastIndexOf(`DeleteRegKey SHCTX "${oldUninstall}"`));
  assert.ok(hook.indexOf(newProduct) < hook.lastIndexOf(`DeleteRegKey SHCTX "${oldProduct}"`));
  assert.match(hook, /IsShortcutTarget[\s\S]*CL-GO\.lnk/);
  assert.doesNotMatch(hook, /\b(?:Exec|ExecWait|nsExec|RMDir)\b/i);
  for (const token of ["..", '"', "/", "*", "?", "<", ">", "|"]) {
    const guard = token === '"' ? `$R8 '"' ">"` : `$R8 "${token}" ">"`;
    assert.ok(hook.includes(guard));
  }
  assert.ok(hook.indexOf("IfFileExists") < hook.indexOf("StrCpy $INSTDIR $R8"));
  assert.ok(hook.indexOf("StrCpy $INSTDIR $R8") < hook.indexOf("SetOutPath $INSTDIR"));

  const deletedKeys = [...hook.matchAll(/DeleteRegKey\s+SHCTX\s+"([^"]+)"/g)].map(
    (match) => match[1],
  );
  assert.deepEqual(deletedKeys, [oldUninstall, oldProduct]);
});

test("la CI exécute le contrat Windows sous les deux moteurs PowerShell", () => {
  const workflow = loadYaml(readBounded(".github/workflows/ci.yml"));
  const validatorSteps = workflow.jobs["backend-windows-native"].steps.filter(
    ({ run }) => run?.includes("check-nsis-migration.test.ps1"),
  );

  assert.deepEqual(
    validatorSteps.map(({ shell }) => shell).sort(),
    ["powershell", "pwsh"],
  );
});
