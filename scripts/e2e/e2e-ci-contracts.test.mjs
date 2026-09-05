import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { load as loadYaml } from "js-yaml";

const readSource = (path) => readFileSync(new URL(path, import.meta.url), "utf8");
const ciSource = readSource("../../.github/workflows/ci.yml");
const mainSource = readSource("../../src-tauri/src/main.rs");
const runnerSource = readSource("./run.mjs");
const wdioSource = readSource("../../wdio.conf.ts");
const macObserverSource = readSource("./macos-app-observer.mjs");
const nativeSmokeSource = readSource("../../tests/e2e/native-cef-shutdown.spec.ts");
const nativeWebViewSource = readSource("../../tests/e2e/native-webview-shutdown.spec.ts");
const invokeSource = readSource("../../src-tauri/src/invoke_handler.rs");
const commandsSource = readSource("../../src-tauri/src/commands/mod.rs");
const e2eCommandSource = readSource("../../src-tauri/src/commands/e2e.rs");
const acceptanceSource = readSource("../../tests/e2e/extensions-ui-acceptance.spec.ts");
const hostSetupSource = readSource("../../tests/e2e/extension-host-setup.ts");
const packagedSource = readSource("./run-packaged.mjs");
const ci = loadYaml(ciSource);

test("CI can target only the native boundary that needs diagnosis", () => {
  assert.deepEqual(ci.on.workflow_dispatch.inputs.target.options, [
    "all",
    "windows-host",
    "windows-backend",
    "windows-native",
    "macos-native",
  ]);
  assert.match(ci.jobs["windows-extension-host-smoke"].if, /windows-host/u);
  assert.match(ci.jobs["backend-windows"].if, /windows-backend/u);
  assert.match(ci.jobs["backend-windows-native"].if, /windows-native/u);
  assert.match(ci.jobs["backend-macos-native"].if, /macos-native/u);
});

test("native Windows CI executes the AppContainer confinement boundary", () => {
  const backend = ci.jobs["backend-windows"];
  const confinement = backend.steps.find(({ name }) => name === "Windows AppContainer confinement");
  assert.ok(confinement);
  assert.match(confinement.run, /appcontainer_writes_only_inside_the_selected_root/u);
  assert.match(confinement.run, /--exact --ignored --nocapture/u);
});

test("Windows multi-command native contracts stop after the first failure", () => {
  const guardedSteps = [
    ["backend-windows", "Windows extension installer preflight"],
    ["backend-windows-native", "Windows CEF supervision authority"],
    ["backend-windows-native", "Ollama manager contracts and historical scenarios"],
    ["windows-extension-host-smoke", "Rust owned-process protocol smoke"],
  ];
  for (const [jobName, stepName] of guardedSteps) {
    const step = ci.jobs[jobName].steps.find(({ name }) => name === stepName);
    assert.equal(step.shell, "bash", `${jobName}: ${stepName}`);
  }
  const ollama = ci.jobs["backend-windows-native"].steps.find(
    ({ name }) => name === "Ollama manager contracts and historical scenarios",
  );
  assert.match(ollama.run, /services::ollama_manager --features windows-tests/u);
});

test("Windows packages and acceptance run in separate jobs", () => {
  const build = ci.jobs["backend-windows-native"];
  const acceptance = ci.jobs["windows-packaged-acceptance"];
  const upload = build.steps.find(({ name }) => name === "Upload Windows E2E package");
  const download = acceptance.steps.find(({ name }) => name === "Download Windows E2E package");
  assert.equal(ci.env.WINDOWS_E2E_PACKAGE_DIR, "target/e2e/debug/bundle/nsis");
  assert.equal(acceptance.needs, "backend-windows-native");
  assert.ok(build.steps.some(({ name }) => name === "Build Windows E2E package"));
  assert.equal(upload.with.path, "${{ env.WINDOWS_E2E_PACKAGE_DIR }}/*.exe");
  assert.equal(download.with.path, "${{ env.WINDOWS_E2E_PACKAGE_DIR }}");
  assert.ok(acceptance.steps.some(({ name }) => name === "Native Windows CEF journey"));
});

test("Windows acceptance always preserves bounded diagnostics and test results", () => {
  const acceptance = ci.jobs["windows-packaged-acceptance"];
  const journey = acceptance.steps.find(({ name }) => name === "Native Windows CEF journey");
  const upload = acceptance.steps.find(({ name }) => name === "Upload Windows E2E diagnostics");
  assert.equal(journey.env.E2E_SKIP_BUILD, "1");
  assert.equal(journey.env.E2E_ARTIFACT_DIR, ".e2e-artifacts/windows-cef");
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(upload.with.path, ".e2e-artifacts/windows-cef");
  assert.equal(upload.with["include-hidden-files"], true);
  assert.equal(upload.with["retention-days"], 14);
});

test("macOS packages and acceptance run in separate jobs", () => {
  const build = ci.jobs["backend-macos-native"];
  const acceptance = ci.jobs["macos-packaged-acceptance"];
  assert.equal(acceptance.needs, "backend-macos-native");
  assert.ok(build.steps.some(({ name }) => name === "Build macOS E2E package"));
  assert.ok(build.steps.some(({ name }) => name === "Archive macOS E2E package"));
  assert.ok(build.steps.some(({ name }) => name === "Upload macOS E2E package"));
  assert.ok(acceptance.steps.some(({ name }) => name === "Download macOS E2E package"));
  assert.ok(acceptance.steps.some(({ name }) => name === "Native macOS CEF journey"));
});

test("macOS acceptance always preserves bounded diagnostics and test results", () => {
  const acceptance = ci.jobs["macos-packaged-acceptance"];
  const journey = acceptance.steps.find(({ name }) => name === "Native macOS CEF journey");
  const upload = acceptance.steps.find(({ name }) => name === "Upload macOS E2E diagnostics");
  assert.equal(journey.env.E2E_SKIP_BUILD, "1");
  assert.equal(journey.env.E2E_ARTIFACT_DIR, ".e2e-artifacts/macos-cef");
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(upload.with.path, ".e2e-artifacts/macos-cef");
  assert.equal(upload.with["include-hidden-files"], true);
  assert.equal(upload.with["retention-days"], 14);
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

test("every E2E failure reports bounded diagnostics before profile cleanup", () => {
  assert.match(runnerSource, /if \(hadPriorFailure \|\| process\.exitCode\) \{/u);
  assert.ok(
    runnerSource.indexOf("await reportNativeDiagnostics(logDirectory)")
      < runnerSource.indexOf("await cleanupProfile(profilePath"),
  );
});

test("CI runs the packaged extension UI journey on all three desktop systems", () => {
  const windowsJob = ciSource.slice(
    ciSource.indexOf("  backend-windows-native:"),
    ciSource.indexOf("  backend-macos-native:"),
  );
  const macJob = ciSource.slice(
    ciSource.indexOf("  backend-macos-native:"),
    ciSource.indexOf("  backend-windows:"),
  );
  const journey = /E2E_REQUIRE_CEF_SMOKE: "1"[\s\S]*npm run test:e2e:packaged(?:\r?\n|$)/u;
  const hostInstall = /npm ci --ignore-scripts --omit=dev --prefix src-tauri\/resources\/extension-host/u;
  assert.match(windowsJob, journey);
  assert.match(macJob, journey);
  assert.match(windowsJob, hostInstall);
  assert.match(macJob, hostInstall);
});

test("CI gates the Windows CEF journey on a real extension load", () => {
  const windowsJob = ciSource.slice(
    ciSource.indexOf("  backend-windows-native:"),
    ciSource.indexOf("  windows-extension-host-smoke:"),
  );
  const smokeJob = ciSource.slice(
    ciSource.indexOf("  windows-extension-host-smoke:"),
    ciSource.indexOf("  backend-macos-native:"),
  );
  assert.match(windowsJob, /needs: windows-extension-host-smoke/u);
  assert.match(smokeJob, /runs-on: windows-latest/u);
  assert.match(smokeJob, /npm run test:extensions-runtime-smoke/u);
  assert.match(
    smokeJob,
    /--filter services::extensions::host_process::prepared_tests::prepared_runtime_[^\r\n]*--features windows-tests --ignored --nocapture/u,
  );
  assert.match(
    smokeJob,
    /prepared_runtime_confirms_restart_stop_while_exit_monitor_is_active --features windows-tests --exact --ignored --nocapture/u,
  );
});

test("CI runs the real Tauri WebView journey on Linux", () => {
  const linuxJob = ciSource.slice(
    ciSource.indexOf("  backend-linux-native:"),
    ciSource.indexOf("  backend-windows-native:"),
  );
  assert.match(
    linuxJob,
    /E2E_REQUIRE_WEBVIEW_SMOKE: "1"[\s\S]*xvfb-run[^\r\n]*npm run test:e2e:packaged(?:\r?\n|$)/u,
  );
  assert.match(linuxJob, /webkit2gtk-driver/u);
});

test("CI installs extension UI journey dependencies", () => {
  const backendJob = ciSource.slice(
    ciSource.indexOf("  backend:"),
    ciSource.indexOf("  backend-linux-native:"),
  );
  const windowsTestsJob = ciSource.slice(
    ciSource.indexOf("  backend-windows:"),
    ciSource.indexOf("  frontend:"),
  );
  const uiBuilderInstall = /Install extension UI builder test dependencies[\s\S]*npm ci --ignore-scripts(?:\r?\n|$)/u;
  assert.match(backendJob, uiBuilderInstall);
  assert.match(windowsTestsJob, uiBuilderInstall);
});

test("the WebDriver journey executes the extension UI runtime proof", () => {
  assert.match(wdioSource, /extensions-ui-runtime-proof\.spec\.ts/u);
  assert.match(runnerSource, /BEAVER_E2E_UI_MANIFEST_SHA/u);
  assert.match(
    wdioSource,
    /extensions-ui-runtime-proof\.spec\.ts[\s\S]*extensions-ui-advanced\.spec\.ts[\s\S]*extensions-ui-acceptance\.spec\.ts/u,
  );
});

test("the packaged journey collects the API expansion acceptance fixture", () => {
  assert.match(wdioSource, /extensions-api-expansion\.spec\.ts/u);
  assert.match(packagedSource, /BEAVER_E2E_API_EXPANSION_FIXTURE/u);
  assert.match(packagedSource, /api-expansion/u);
  assert.match(packagedSource, /beaver-extension\.json/u);
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
  assert.match(invokeSource, /#\[cfg\((?:all\()?feature = "e2e"[\s\S]*e2e_request_exit/u);
  assert.match(invokeSource, /#\[cfg\((?:all\()?not\(feature = "e2e"\)/u);
});

test("installed extension UI acceptance opts into only its host", () => {
  assert.match(e2eCommandSource, /pub fn e2e_initialize_extension_host/u);
  assert.match(
    invokeSource,
    /#\[cfg\((?:all\()?feature = "e2e"[\s\S]*e2e_initialize_extension_host/u,
  );
  assert.match(acceptanceSource, /initializeExtensionHost\(\)/u);
  assert.match(hostSetupSource, /invokeTauri\("e2e_initialize_extension_host"\)/u);
  assert.match(acceptanceSource, /get_extension_host_status/u);
});

test("installed extension UI setup exposes each bounded failure phase", () => {
  for (const phase of [
    "initializes the extension host",
    "installs the standard extension fixture",
    "installs the theme extension fixture",
    "installs the advanced extension fixture",
  ]) {
    assert.match(acceptanceSource, new RegExp(`before\\("${phase}"`, "u"));
  }
  assert.match(acceptanceSource, /this\.timeout\(EXTENSION_SETUP_TIMEOUT_MS\)/u);
});

test("the native smoke releases WebDriver before coordinated exit", () => {
  const request = nativeSmokeSource.indexOf('invokeTauri("e2e_request_exit")');
  const release = nativeSmokeSource.indexOf("browser.deleteSession()", request);
  const detach = nativeSmokeSource.indexOf("sessionId = undefined", release);
  const observation = nativeSmokeSource.indexOf("waitForOwnedProcessesToExit", detach);
  assert.ok(request >= 0 && release > request && detach > release && observation > detach);
});
