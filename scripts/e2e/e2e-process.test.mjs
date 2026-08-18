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

const ciSource = readFileSync(new URL("../../.github/workflows/ci.yml", import.meta.url), "utf8");
const mainSource = readFileSync(new URL("../../src-tauri/src/main.rs", import.meta.url), "utf8");
const runnerSource = readFileSync(new URL("./run.mjs", import.meta.url), "utf8");
const packagedRunnerSource = readFileSync(
  new URL("./run-packaged.mjs", import.meta.url),
  "utf8",
);
const wdioSource = readFileSync(new URL("../../wdio.conf.ts", import.meta.url), "utf8");
const macObserverSource = readFileSync(new URL("./macos-app-observer.mjs", import.meta.url), "utf8");
const nativeSmokeSource = readFileSync(
  new URL("../../tests/e2e/native-cef-shutdown.spec.ts", import.meta.url),
  "utf8",
);
const nativeWebViewSource = readFileSync(
  new URL("../../tests/e2e/native-webview-shutdown.spec.ts", import.meta.url),
  "utf8",
);
const invokeSource = readFileSync(
  new URL("../../src-tauri/src/invoke_handler.rs", import.meta.url),
  "utf8",
);
const commandsSource = readFileSync(
  new URL("../../src-tauri/src/commands/mod.rs", import.meta.url),
  "utf8",
);

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
    "src-tauri/tauri.e2e.conf.json", "--bundles", "nsis",
  ]);
});

test("the packaged runner selects the packaged binary before WebDriver", () => {
  assert.match(packagedRunnerSource, /process\.env\.E2E_PACKAGED = "1"/u);
  assert.match(packagedRunnerSource, /await import\("\.\/run\.mjs"\)/u);
  const preparation = runnerSource.indexOf("await preparePackagedApp({");
  const webdriver = runnerSource.indexOf("node_modules/@wdio/cli/bin/wdio.js");
  assert.ok(preparation >= 0 && webdriver > preparation);
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
  assert.equal(E2E_BUILD_TIMEOUT_MS, 35 * 60 * 1000);
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

test("CI exercises Rust assertions and clippy with the E2E feature", () => {
  assert.match(ciSource, /cargo clippy --all-targets --features e2e -- -D warnings/u);
  assert.match(ciSource, /cargo test --all --features e2e/u);
});

test("release builds reject the E2E control feature in the application binary", () => {
  assert.match(
    mainSource,
    /cfg\(all\(feature = "e2e", not\(debug_assertions\)\)\)[\s\S]*compile_error!/u,
  );
});

test("profile cleanup cannot hide the preceding E2E failure", () => {
  assert.match(runnerSource, /cleanupProfile\(profilePath,[\s\S]*hadPriorFailure/u);
});

test("every E2E failure reports bounded diagnostics before profile cleanup", () => {
  assert.match(runnerSource, /if \(hadPriorFailure \|\| process\.exitCode\) \{/u);
  assert.ok(
    runnerSource.indexOf("await reportNativeDiagnostics(logDirectory)")
      < runnerSource.indexOf("await cleanupProfile(profilePath"),
  );
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

test("CI runs the real CEF journey on Windows and macOS only", () => {
  const windowsJob = ciSource.slice(
    ciSource.indexOf("  backend-windows-native:"),
    ciSource.indexOf("  backend-macos-native:"),
  );
  const macJob = ciSource.slice(
    ciSource.indexOf("  backend-macos-native:"),
    ciSource.indexOf("  backend-windows:"),
  );
  assert.match(windowsJob, /E2E_REQUIRE_CEF_SMOKE: "1"[\s\S]*npm run test:e2e:packaged/u);
  assert.match(macJob, /E2E_REQUIRE_CEF_SMOKE: "1"[\s\S]*npm run test:e2e:packaged/u);
  const extensionHostInstall = /npm ci --ignore-scripts --omit=dev --prefix src-tauri\/resources\/extension-host/u;
  assert.match(windowsJob, extensionHostInstall);
  assert.match(macJob, extensionHostInstall);
});

test("CI runs the real Tauri WebView journey on Linux", () => {
  const linuxJob = ciSource.slice(
    ciSource.indexOf("  backend-linux-native:"),
    ciSource.indexOf("  backend-windows-native:"),
  );
  assert.match(linuxJob, /E2E_REQUIRE_WEBVIEW_SMOKE: "1"[\s\S]*xvfb-run[\s\S]*npm run test:e2e/u);
  assert.match(linuxJob, /webkit2gtk-driver/u);
});

test("the native CEF journey uses one isolated application session", () => {
  assert.match(wdioSource, /E2E_REQUIRE_CEF_SMOKE[\s\S]*native-cef-shutdown\.spec\.ts[\s\S]*onboarding\.spec\.ts/u);
  assert.match(nativeSmokeSource, /completeOnboarding\(\)/u);
  assert.match(wdioSource, /logLevel:\s*nativeSmoke\s*\?\s*"info"\s*:\s*"warn"/u);
  assert.match(runnerSource, /const logDirectory = join\(profilePath,\s*"logs"\)/u);
  assert.match(runnerSource, /E2E_LOG_DIR:\s*logDirectory/u);
  assert.match(wdioSource, /outputDir:\s*e2eLogDirectory/u);
  assert.match(wdioSource, /process\.platform === "darwin"[\s\S]*macos-app-observer\.mjs/u);
  assert.match(wdioSource, /appBinaryPath:\s*driverBinaryPath/u);
  assert.match(wdioSource, /appArgs:\s*driverArguments/u);
});

test("macOS captures CEF helper turnover before launching Beaver", () => {
  const capture = macObserverSource.indexOf("captureMacCefTurnoverProof(");
  const spawn = macObserverSource.indexOf("spawn(launch.command", capture);
  assert.ok(capture >= 0 && spawn > capture);
  assert.match(nativeSmokeSource, /waitForMacCefTurnoverProof/u);
});

test("the native WebView journey observes classified pids before coordinated exit", () => {
  const observation = nativeWebViewSource.indexOf('invokeTauri<NativeWebViews>("e2e_native_webviews")');
  const request = nativeWebViewSource.indexOf('invokeTauri("e2e_request_exit")', observation);
  const release = nativeWebViewSource.indexOf("browser.deleteSession()", request);
  const exit = nativeWebViewSource.indexOf("waitForProcessIdsToExit", release);
  assert.ok(observation >= 0 && request > observation && release > request && exit > release);
  assert.match(wdioSource, /E2E_REQUIRE_WEBVIEW_SMOKE[\s\S]*native-webview-shutdown\.spec\.ts/u);
});

test("the coordinated exit command is compiled only into the E2E handler", () => {
  assert.match(commandsSource, /#\[cfg\(feature = "e2e"\)\][\s\S]*mod e2e/u);
  assert.match(invokeSource, /#\[cfg\(feature = "e2e"\)\][\s\S]*e2e_request_exit/u);
  assert.match(invokeSource, /#\[cfg\(not\(feature = "e2e"\)\)\]/u);
});

test("the native smoke releases WebDriver before Beaver performs its coordinated exit", () => {
  const request = nativeSmokeSource.indexOf('invokeTauri("e2e_request_exit")');
  const release = nativeSmokeSource.indexOf("browser.deleteSession()", request);
  const detach = nativeSmokeSource.indexOf("sessionId = undefined", release);
  const observation = nativeSmokeSource.indexOf("waitForOwnedProcessesToExit", detach);
  assert.ok(request >= 0 && release > request && detach > release && observation > detach);
});
