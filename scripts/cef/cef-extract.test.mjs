import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, join, resolve } from "node:path";
import test from "node:test";
import * as cefExtract from "./cef-extract.mjs";

test("Windows CEF layout is flattened for cef-dll-sys", async (context) => {
  assert.equal(typeof cefExtract.normalizeWindowsCefLayout, "function");
  if (typeof cefExtract.normalizeWindowsCefLayout !== "function") return;

  const root = await mkdtemp(join(tmpdir(), "clgo-cef-layout-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(join(root, "Release"));
  await mkdir(join(root, "Resources", "locales"), { recursive: true });
  await writeFile(join(root, "Release", "libcef.dll"), "runtime");
  await writeFile(join(root, "Resources", "resources.pak"), "resources");
  await writeFile(join(root, "Resources", "locales", "en-US.pak"), "locale");
  await writeFile(join(root, "CMakeLists.txt"), "cmake");

  await cefExtract.normalizeWindowsCefLayout(root);

  await assert.rejects(() => rm(join(root, "Release")));
  await assert.rejects(() => rm(join(root, "Resources")));
  assert.equal(await read(join(root, "libcef.dll")), "runtime");
  assert.equal(await read(join(root, "resources.pak")), "resources");
  assert.equal(await read(join(root, "locales", "en-US.pak")), "locale");
  assert.equal(await read(join(root, "CMakeLists.txt")), "cmake");
});

async function read(path) {
  return import("node:fs/promises").then(({ readFile }) => readFile(path, "utf8"));
}

test("an old Windows CEF layout marker is invalidated", async (context) => {
  assert.equal(typeof cefExtract.isCurrentCefLayout, "function");
  if (typeof cefExtract.isCurrentCefLayout !== "function") return;

  const current = await mkdtemp(join(tmpdir(), "clgo-cef-current-"));
  context.after(() => rm(current, { recursive: true, force: true }));
  await writeFile(join(current, ".clgo-sha256"), "pinned-hash\n");
  await writeFile(join(current, "CMakeLists.txt"), "cmake");

  assert.equal(
    await cefExtract.isCurrentCefLayout(
      current,
      { sha256: "pinned-hash" },
      "win32",
    ),
    false,
  );
});

test("Windows CEF extraction retries one transient failure", async () => {
  const { extractCefWithRetry } = await import("./cef-prepare-retry.mjs");
  let attempts = 0;
  const extract = async () => {
    attempts += 1;
    if (attempts === 1) throw new Error("transient failure");
    return "ready";
  };

  const result = await extractCefWithRetry("archive", {}, {
    extract,
    platform: "win32",
    wait: async () => {},
  });

  assert.equal(result, "ready");
  assert.equal(attempts, 2);
});

test("CEF extraction remains fail closed after its Windows retry", async () => {
  const { extractCefWithRetry } = await import("./cef-prepare-retry.mjs");
  let attempts = 0;
  const extract = async () => {
    attempts += 1;
    throw new Error("persistent failure");
  };

  await assert.rejects(
    extractCefWithRetry("archive", {}, {
      extract,
      platform: "win32",
      wait: async () => {},
    }),
    /persistent failure/,
  );
  assert.equal(attempts, 2);
});

test("the Tauri launcher exposes the verified Ninja directory first", async () => {
  const { repoRoot } = await import("./cef-artifacts.mjs");
  const { createTauriLaunch } = await import("./tauri-launch.mjs");
  const launch = createTauriLaunch({
    args: ["dev"],
    cliPath: join("project", "node_modules", "@tauri-apps", "cli", "tauri.js"),
    currentPath: join("Windows", "System32"),
    executablePath: join("node", "node.exe"),
    toolPath: join("project", "src-tauri", ".cef-tools", "ninja.exe"),
  });

  assert.equal(launch.command, join("node", "node.exe"));
  assert.deepEqual(launch.args, [
    join("project", "node_modules", "@tauri-apps", "cli", "tauri.js"),
    "dev",
  ]);
  assert.equal(
    launch.path,
    `${join("project", "src-tauri", ".cef-tools")}${delimiter}${join("Windows", "System32")}`,
  );
  const packageJson = JSON.parse(await read(join(repoRoot, "package.json")));
  assert.equal(packageJson.scripts.tauri, "node scripts/cef/run-tauri.mjs");
});

test("the Tauri launcher shortens the default Windows Cargo target path", async () => {
  const { resolveCargoTargetDir } = await import("./tauri-launch.mjs");
  const projectRoot = join("workspace", "project");

  assert.equal(typeof resolveCargoTargetDir, "function");
  assert.equal(
    resolveCargoTargetDir({
      configuredTargetDir: undefined,
      platform: "win32",
      repoRoot: projectRoot,
    }),
    resolve(projectRoot, "target"),
  );
});

test("the Tauri launcher preserves a configured Cargo target path", async () => {
  const { resolveCargoTargetDir } = await import("./tauri-launch.mjs");
  const configuredTargetDir = join("cache", "custom-cargo-target");

  assert.equal(typeof resolveCargoTargetDir, "function");
  assert.equal(
    resolveCargoTargetDir({
      configuredTargetDir,
      platform: "win32",
      repoRoot: join("workspace", "project"),
    }),
    configuredTargetDir,
  );
});

test("the Tauri launcher keeps the native Cargo target path outside Windows", async () => {
  const { resolveCargoTargetDir } = await import("./tauri-launch.mjs");

  assert.equal(typeof resolveCargoTargetDir, "function");
  assert.equal(
    resolveCargoTargetDir({
      configuredTargetDir: undefined,
      platform: "darwin",
      repoRoot: join("workspace", "project"),
    }),
    undefined,
  );
});

test("the Tauri launcher rejects an oversized resolved Cargo target path", async () => {
  const { resolveCargoTargetDir } = await import("./tauri-launch.mjs");

  assert.throws(
    () =>
      resolveCargoTargetDir({
        configuredTargetDir: undefined,
        platform: "win32",
        repoRoot: "x".repeat(30_000),
      }),
    /invalid/,
  );
});

test("the Tauri launcher rejects an unbounded argument list", async () => {
  const { createTauriLaunch } = await import("./tauri-launch.mjs");

  assert.throws(
    () =>
      createTauriLaunch({
        args: Array.from({ length: 65 }, () => "dev"),
        cliPath: "tauri.js",
        currentPath: "system",
        executablePath: "node.exe",
        toolPath: "ninja.exe",
      }),
    /Tauri launch configuration is invalid/,
  );
});
