import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { mkdir, mkdtemp, realpath, rm, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "node:test";
import * as e2eProcess from "./e2e-process.mjs";

const {
  buildArguments,
  cleanupProfile,
  debugBinaryPath,
  E2E_BUILD_TIMEOUT_MS,
  E2E_JOURNEY_TIMEOUT_MS,
  e2eCargoTargetDir,
  isAllowedProfilePath,
  runCommand,
} = e2eProcess;

const runnerSource = readFileSync(new URL("./run.mjs", import.meta.url), "utf8");
const buildPreparationSource = readFileSync(
  new URL("./prepare-build.mjs", import.meta.url),
  "utf8",
);
const packagedRunnerSource = readFileSync(
  new URL("./run-packaged.mjs", import.meta.url),
  "utf8",
);
const baseE2eConfig = JSON.parse(readFileSync(
  new URL("../../src-tauri/tauri.e2e.conf.json", import.meta.url),
  "utf8",
));
const windowsE2eConfig = JSON.parse(readFileSync(
  new URL("../../src-tauri/tauri.e2e.windows.conf.json", import.meta.url),
  "utf8",
));

test("the E2E build always enables the isolated feature", () => {
  assert.deepEqual(buildArguments("linux"), [
    "build", "--debug", "--features", "e2e", "--config",
    "src-tauri/tauri.e2e.conf.json", "--no-bundle",
  ]);
  assert.deepEqual(buildArguments("darwin"), [
    "build", "--debug", "--features", "e2e", "--config",
    "src-tauri/tauri.e2e.conf.json", "--bundles", "app",
  ]);
  assert.deepEqual(buildArguments("win32"), [
    "build", "--debug", "--features", "e2e", "--config",
    "src-tauri/tauri.e2e.conf.json", "--no-bundle",
  ]);
  assert.deepEqual(buildArguments("win32", true), [
    "build", "--debug", "--features", "e2e", "--config",
    "src-tauri/tauri.e2e.conf.json", "--config",
    "src-tauri/tauri.e2e.windows.conf.json", "--bundles", "nsis",
  ]);
  assert.deepEqual(buildArguments("linux", true), [
    "build", "--debug", "--features", "e2e", "--config",
    "src-tauri/tauri.e2e.conf.json", "--bundles", "appimage",
  ]);
  assert.deepEqual(buildArguments("darwin", true), [
    "build", "--debug", "--features", "e2e", "--config",
    "src-tauri/tauri.e2e.conf.json", "--bundles", "app",
  ]);
});

test("the E2E build prepares and bundles the real extension host", () => {
  assert.match(buildPreparationSource, /prepare-extension-host\.mjs/u);
  assert.equal(
    baseE2eConfig.bundle.resources["resources/extension-host/"],
    "resources/extension-host/",
  );
});

test("the Windows packaged build has an isolated product identity", () => {
  assert.equal(baseE2eConfig.identifier, "com.clgo.dash.e2e");
  assert.equal(windowsE2eConfig.productName, "Beaver E2E");
  assert.equal(windowsE2eConfig.identifier, undefined);
  assert.equal(windowsE2eConfig.bundle.windows.nsis.installerHooks, null);
});

test("the packaged runner selects the packaged binary before WebDriver", () => {
  assert.match(packagedRunnerSource, /process\.env\.E2E_PACKAGED = "1"/u);
  assert.match(packagedRunnerSource, /await import\("\.\/run\.mjs"\)/u);
  const preparation = runnerSource.indexOf("await preparePackagedApp({");
  const webdriver = runnerSource.indexOf("node_modules/@wdio/cli/bin/wdio.js");
  assert.ok(preparation >= 0 && webdriver > preparation);
});

test("the E2E runner separates package construction from acceptance", () => {
  assert.equal(typeof e2eProcess.resolveE2eRunMode, "function");
  if (typeof e2eProcess.resolveE2eRunMode !== "function") return;
  assert.deepEqual(e2eProcess.resolveE2eRunMode({}), {
    build: true,
    journey: true,
  });
  assert.deepEqual(e2eProcess.resolveE2eRunMode({ E2E_BUILD_ONLY: "1" }), {
    build: true,
    journey: false,
  });
  assert.deepEqual(e2eProcess.resolveE2eRunMode({ E2E_SKIP_BUILD: "1" }), {
    build: false,
    journey: true,
  });
  assert.throws(
    () => e2eProcess.resolveE2eRunMode({ E2E_BUILD_ONLY: "1", E2E_SKIP_BUILD: "1" }),
    /E2E run mode is invalid/u,
  );
});

test("the E2E binary path is platform specific", () => {
  const cargoTargetDir = resolve("/repo", "target", "e2e");
  const debugRoot = resolve(cargoTargetDir, "debug");

  assert.equal(debugBinaryPath("linux", cargoTargetDir), join(debugRoot, "cl-go-dash"));
  assert.equal(debugBinaryPath("win32", cargoTargetDir), join(debugRoot, "cl-go-dash.exe"));
  assert.equal(
    debugBinaryPath("darwin", cargoTargetDir),
    join(debugRoot, "bundle", "macos", "Beaver.app", "Contents", "MacOS", "cl-go-dash"),
  );
});

test("the E2E build and binary reader share one Cargo target directory", () => {
  const projectRoot = resolve("workspace", "project");
  const configured = resolve("cache", "beaver-e2e");

  assert.equal(
    e2eCargoTargetDir("win32", projectRoot, undefined),
    resolve(projectRoot, "target", "e2e"),
  );
  assert.equal(
    e2eCargoTargetDir("darwin", projectRoot, undefined),
    resolve(projectRoot, "src-tauri", "target", "e2e"),
  );
  assert.equal(e2eCargoTargetDir("win32", projectRoot, configured), configured);
});

test("the E2E repository root resolves through a symbolic link", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-e2e-root-"));
  const physicalRoot = join(directory, "repository");
  const linkedRoot = join(directory, "checkout");
  try {
    await mkdir(physicalRoot);
    await symlink(
      physicalRoot,
      linkedRoot,
      process.platform === "win32" ? "junction" : "dir",
    );
    const moduleUrl = pathToFileURL(
      join(linkedRoot, "scripts", "e2e", "run.mjs"),
    ).href;

    assert.equal(typeof e2eProcess.canonicalE2eRepoRoot, "function");
    assert.equal(
      await e2eProcess.canonicalE2eRepoRoot(moduleUrl),
      await realpath(physicalRoot),
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("a cold native build has a larger budget than the bounded app journey", () => {
  assert.equal(E2E_BUILD_TIMEOUT_MS, 55 * 60 * 1000);
  assert.equal(E2E_JOURNEY_TIMEOUT_MS, 10 * 60 * 1000);
  assert.ok(E2E_BUILD_TIMEOUT_MS > E2E_JOURNEY_TIMEOUT_MS);
});

test("runCommand fails when its bounded process exceeds the selected budget", async () => {
  const exitCode = await runCommand(
    process.execPath,
    ["-e", "setInterval(() => {}, 1_000)"],
    {
      cwd: process.cwd(),
      env: process.env,
      timeoutMs: 50,
    },
  );

  assert.equal(exitCode, 1);
});

test("only a dedicated direct child of the system temp directory is accepted", () => {
  assert.equal(isAllowedProfilePath("/tmp/beaver-e2e-Ab12", "/tmp"), true);
  assert.equal(isAllowedProfilePath("/tmp/beaver-e2e-Ab12/nested", "/tmp"), false);
  assert.equal(isAllowedProfilePath("/tmp/another-profile", "/tmp"), false);
  assert.equal(isAllowedProfilePath("/repo", "/tmp"), false);
});

test("profile cleanup cannot hide the preceding E2E failure", () => {
  assert.match(runnerSource, /cleanupProfile\(profilePath,[\s\S]*hadPriorFailure/u);
});

test("profile cleanup preserves an earlier E2E failure", async () => {
  const tempRoot = resolve("temp-root");
  const profilePath = join(tempRoot, "beaver-e2e-Ab12");
  const reports = [];
  await cleanupProfile(profilePath, {
    tempPath: tempRoot,
    hadPriorFailure: true,
    remove: async () => { throw new Error("locked"); },
    report: (message) => reports.push(message),
  });

  assert.deepEqual(reports, ["E2E profile cleanup failed after an earlier failure.\n"]);
  await assert.rejects(
    cleanupProfile(profilePath, {
      tempPath: tempRoot,
      hadPriorFailure: false,
      remove: async () => { throw new Error("locked"); },
    }),
    /E2E profile cleanup failed/u,
  );
});

test("profile cleanup rejects a target outside the isolated E2E shape", async () => {
  const tempRoot = resolve("temp-root");
  let removed = false;

  await assert.rejects(
    cleanupProfile(tempRoot, {
      tempPath: tempRoot,
      remove: async () => { removed = true; },
    }),
    /E2E profile cleanup failed/u,
  );
  assert.equal(removed, false);
});
