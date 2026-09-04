import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { mkdir, mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { prepareRelease } from "./prepare-release.mjs";

async function recordPreparation(platform) {
  const root = await realpath(await mkdtemp(join(tmpdir(), "beaver-release-")));
  const calls = [];
  const record = (name) => async ({ repoRoot }) => {
    assert.equal(repoRoot, root);
    calls.push(name);
  };
  try {
    await prepareRelease({
      repoRoot: root,
      platform,
      prepareExtensionUi: record("extension-ui"),
      prepareExtensions: record("extensions"),
      prepareCefSource: record("cef-source"),
      buildFrontend: record("frontend"),
      prepareUpdater: record("updater"),
      prepareSearxng: record("searxng"),
      prepareUnixCef: record("unix-cef"),
    });
    return calls;
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

test("prépare Windows sans lancer de script Bash", async () => {
  assert.deepEqual(await recordPreparation("win32"), [
    "extension-ui",
    "extensions",
    "cef-source",
    "frontend",
    "updater",
    "searxng",
  ]);
});

test("Tauri utilise uniquement la préparation native centralisée", () => {
  const config = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
  assert.equal(config.build.beforeBuildCommand, "node scripts/build/prepare-release.mjs");
  assert.equal(existsSync("src-tauri/scripts/prepare-updater-helper.sh"), false);
  assert.equal(existsSync("src-tauri/scripts/prepare-searxng.sh"), false);
});

test("la CSP complète interdit les sources de scripts non approuvées", () => {
  const config = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
  assert.equal(
    config.app.security.csp,
    "default-src 'self'; style-src 'self' 'unsafe-inline' beaver-extension: http://beaver-extension.localhost; script-src 'self' beaver-extension: http://beaver-extension.localhost; img-src 'self' data: https: beaver-extension: http://beaver-extension.localhost; font-src 'self' data: beaver-extension: http://beaver-extension.localhost",
  );
});

test("conserve la préparation CEF Unix après les étapes communes", async () => {
  assert.deepEqual(await recordPreparation("linux"), [
    "extension-ui",
    "extensions",
    "cef-source",
    "frontend",
    "updater",
    "searxng",
    "unix-cef",
  ]);
});

test("lance la préparation CEF Unix depuis le dossier Tauri", async () => {
  const root = await realpath(await mkdtemp(join(tmpdir(), "beaver-release-")));
  const tauriDir = join(root, "src-tauri");
  const script = join(tauriDir, "scripts", "prepare-cef.sh");
  const commands = [];
  const skip = async () => {};
  try {
    await mkdir(join(tauriDir, "scripts"), { recursive: true });
    await writeFile(script, "#!/usr/bin/env bash\n", "utf8");
    await prepareRelease({
      repoRoot: root,
      platform: "linux",
      prepareExtensionUi: skip,
      prepareExtensions: skip,
      prepareCefSource: skip,
      buildFrontend: skip,
      prepareUpdater: skip,
      prepareSearxng: skip,
      run: async (command) => commands.push(command),
    });
    assert.deepEqual(commands, [{ command: "bash", args: [script], cwd: tauriDir }]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("échoue fermée pour une plateforme ou des étapes invalides", async () => {
  await assert.rejects(
    () => prepareRelease({ repoRoot: process.cwd(), platform: "windows" }),
    /Release preparation failed/,
  );
});
