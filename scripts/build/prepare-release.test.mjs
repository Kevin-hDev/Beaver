import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { mkdtemp, realpath, rm } from "node:fs/promises";
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

test("conserve la préparation CEF Unix après les étapes communes", async () => {
  assert.deepEqual(await recordPreparation("linux"), [
    "extensions",
    "cef-source",
    "frontend",
    "updater",
    "searxng",
    "unix-cef",
  ]);
});

test("échoue fermée pour une plateforme ou des étapes invalides", async () => {
  await assert.rejects(
    () => prepareRelease({ repoRoot: process.cwd(), platform: "windows" }),
    /Release preparation failed/,
  );
});
