import { resolve } from "node:path";
import { NATIVE_JOURNEY_MOCHA_TIMEOUT_MS } from "./scripts/e2e/native-journey-deadline.mjs";

const appBinaryPath = process.env.E2E_APP_BINARY;
if (!appBinaryPath) throw new Error("E2E app binary is not configured");
const e2eLogDirectory = process.env.E2E_LOG_DIR;
if (!e2eLogDirectory) throw new Error("E2E log directory is not configured");
const nativeCefSmoke = process.env.E2E_REQUIRE_CEF_SMOKE === "1";
const nativeWebViewSmoke = process.env.E2E_REQUIRE_WEBVIEW_SMOKE === "1";
const nativeSmoke = nativeCefSmoke || nativeWebViewSmoke;
const childSessionReadOnlySpec = "./tests/e2e/child-session-read-only.spec.ts";
const shutdownSpec = nativeCefSmoke
  ? "./tests/e2e/native-cef-shutdown.spec.ts"
  : nativeWebViewSmoke
    ? "./tests/e2e/native-webview-shutdown.spec.ts"
    : "./tests/e2e/onboarding.spec.ts";
const observeMacApplication = process.platform === "darwin";
const driverBinaryPath = observeMacApplication ? process.execPath : appBinaryPath;
const driverArguments = observeMacApplication
  ? [resolve(process.cwd(), "scripts/e2e/macos-app-observer.mjs")]
  : undefined;

export const config: WebdriverIO.Config = {
  outputDir: e2eLogDirectory,
  runner: "local",
  specs: [[shutdownSpec, childSessionReadOnlySpec]],
  maxInstances: 1,
  capabilities: [{ browserName: "tauri" }],
  services: [["@wdio/tauri-service", {
    appBinaryPath: driverBinaryPath,
    appArgs: driverArguments,
    driverProvider: "embedded",
    captureBackendLogs: true,
    captureFrontendLogs: true,
  }]],
  framework: "mocha",
  reporters: ["spec"],
  logLevel: nativeSmoke ? "info" : "warn",
  bail: 1,
  waitforTimeout: 15_000,
  connectionRetryTimeout: 90_000,
  connectionRetryCount: 1,
  mochaOpts: { ui: "bdd", timeout: NATIVE_JOURNEY_MOCHA_TIMEOUT_MS },
};
