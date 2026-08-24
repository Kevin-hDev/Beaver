import assert from "node:assert/strict";
import { readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

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

test("les validateurs natifs de paquets sont présents et bornés", () => {
  const ci = readBounded(".github/workflows/ci.yml");
  const deb = readBounded("scripts/release/check-deb-migration.sh");
  const nsis = readBounded("scripts/release/check-nsis-migration.ps1");
  const windowsHelpers = readBounded(
    "scripts/release/windows-artifact-helpers.ps1",
  );

  assert.match(deb, /dpkg-deb/);
  assert.match(deb, /MAX_CONTENT_ENTRIES=20000/);
  assert.match(deb, /NR > max_entries/);
  assert.match(deb, /Package.*beaver/);
  assert.match(deb, /usr\/bin\/cl-go-dash/);
  assert.match(nsis, /Get-ItemProperty/);
  assert.match(nsis, /Windows package check failed: \$Code/);
  assert.match(nsis, /\[ValidateSet\([\s\S]*installed-shortcuts[\s\S]*\)\]/);
  assert.doesNotMatch(nsis, /Stop-Validation\s*(?:\r?\n|$)/);
  assert.match(nsis, /cl-go-dash\.exe/);
  assert.match(nsis, /target\\updater-helper\\cl-go-dash-updater\.exe/);
  assert.match(nsis, /MaxUpdaterHelperBytes/);
  assert.match(nsis, /windows-artifact-helpers\.ps1/);
  assert.match(nsis, /Test-FullyQualifiedWindowsPath/);
  assert.match(nsis, /Test-BeaverShortcutState/);
  assert.match(nsis, /Test-UpdaterHelper/);
  assert.match(windowsHelpers, /function Test-FullyQualifiedWindowsPath/);
  assert.match(windowsHelpers, /function Test-BeaverShortcutState/);
  assert.match(windowsHelpers, /function Test-UpdaterHelper/);
  assert.match(windowsHelpers, /function Get-VisibleBitmapPixelHash/);
  assert.match(windowsHelpers, /function Get-RenderedIconPixelHashes/);
  assert.match(windowsHelpers, /\$ExpectedIconPath/);
  assert.match(windowsHelpers, /Get-RenderedIconPixelHashes \$actualIcon/);
  assert.match(windowsHelpers, /Get-RenderedIconPixelHashes \$expectedIcon/);
  assert.match(windowsHelpers, /\$actualHashes\[0\] -ceq \$expectedHashes\[0\]/);
  assert.match(windowsHelpers, /\$actualHashes\[1\] -ceq \$expectedHashes\[1\]/);
  assert.match(nsis, /src-tauri\/icons\/icon\.ico/);
  assert.match(
    nsis,
    /Test-BeaverExecutableBrand \$binary \$expectedVersion \$expectedIcon/,
  );
  assert.doesNotMatch(windowsHelpers, /expectedIconSha256/);
  assert.match(nsis, /\.IndexOf\(\$value, \[StringComparison\]::OrdinalIgnoreCase\) -ge 0/);
  assert.doesNotMatch(nsis, /\.Contains\(\$value, \[StringComparison\]/);
  assert.doesNotMatch(nsis, /IsPathFullyQualified/);
  assert.match(
    readBounded("scripts/test-install-ps1.ps1"),
    /check-nsis-migration\.test\.ps1/,
  );
  assert.match(ci, /name: Test Windows package validator[\s\S]*check-nsis-migration\.test\.ps1/);
  for (const variable of [
    "oldUninstall",
    "newUninstall",
    "oldProduct",
    "newProduct",
  ]) {
    assert.match(
      nsis,
      new RegExp(`\\$${variable} = @\\(Get-ExistingRegistryPaths `),
    );
  }
  assert.doesNotMatch(`${nsis}\n${windowsHelpers}`, /Invoke-Expression|cmd\.exe/i);
});
