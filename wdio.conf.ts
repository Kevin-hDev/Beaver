import { resolve } from "node:path";
import {
  EXTENSION_HOST_SETUP_TIMEOUT_MS,
  EXTENSION_DIAGNOSTIC_TIMEOUT_MS,
  EXTENSION_UI_SETUP_TIMEOUT_MS,
  WEBDRIVER_IMPLICIT_TIMEOUT_MS,
  WEBDRIVER_PAGE_LOAD_TIMEOUT_MS,
} from "./scripts/e2e/extension-setup-deadline";
import { NATIVE_JOURNEY_MOCHA_TIMEOUT_MS } from "./scripts/e2e/native-journey-deadline.mjs";

const appBinaryPath = process.env.E2E_APP_BINARY;
if (!appBinaryPath) throw new Error("E2E app binary is not configured");
const e2eLogDirectory = process.env.E2E_LOG_DIR;
if (!e2eLogDirectory) throw new Error("E2E log directory is not configured");
const nativeCefSmoke = process.env.E2E_REQUIRE_CEF_SMOKE === "1";
const nativeWebViewSmoke = process.env.E2E_REQUIRE_WEBVIEW_SMOKE === "1";
const nativeSmoke = nativeCefSmoke || nativeWebViewSmoke;
const artifactDirectory = process.env.E2E_ARTIFACT_DIR;
const activationDiagnostic = process.env.E2E_EXTENSION_ACTIVATION_DIAGNOSTIC === "1";
const childSessionReadOnlySpec = "./tests/e2e/child-session-read-only.spec.ts";
const extensionUiRuntimeProofSpec = "./tests/e2e/extensions-ui-runtime-proof.spec.ts";
const extensionUiAdvancedSpec = "./tests/e2e/extensions-ui-advanced.spec.ts";
const extensionUiAcceptanceSpec = "./tests/e2e/extensions-ui-acceptance.spec.ts";
const extensionApiExpansionSpec = "./tests/e2e/extensions-api-expansion.spec.ts";
const journeySpec = nativeCefSmoke
  ? "./tests/e2e/native-cef-shutdown.spec.ts"
  : nativeWebViewSmoke
    ? "./tests/e2e/native-webview-shutdown.spec.ts"
    : "./tests/e2e/onboarding.spec.ts";
const observeMacApplication = process.platform === "darwin";
const driverBinaryPath = observeMacApplication ? process.execPath : appBinaryPath;
const driverArguments = observeMacApplication
  ? [resolve(process.cwd(), "scripts/e2e/macos-app-observer.mjs")]
  : undefined;
const mochaTimeoutMs = Math.max(
  NATIVE_JOURNEY_MOCHA_TIMEOUT_MS,
  EXTENSION_UI_SETUP_TIMEOUT_MS,
);

export const config: WebdriverIO.Config = {
  outputDir: e2eLogDirectory,
  runner: "local",
  specs: [[
    childSessionReadOnlySpec,
    extensionUiRuntimeProofSpec,
    extensionUiAdvancedSpec,
    ...(activationDiagnostic
      ? ["./tests/e2e/extensions-activation-diagnostic.spec.ts"]
      : [extensionUiAcceptanceSpec, extensionApiExpansionSpec]),
    journeySpec,
  ]],
  maxInstances: 1,
  capabilities: [{
    browserName: "tauri",
    // Let Beaver's bounded host request report its precise failure before the
    // WebDriver script guard can replace it with a generic timeout.
    timeouts: {
      implicit: WEBDRIVER_IMPLICIT_TIMEOUT_MS,
      pageLoad: WEBDRIVER_PAGE_LOAD_TIMEOUT_MS,
      script: EXTENSION_HOST_SETUP_TIMEOUT_MS,
    },
  }],
  services: [["@wdio/tauri-service", {
    appBinaryPath: driverBinaryPath,
    appArgs: driverArguments,
    driverProvider: "embedded",
    captureBackendLogs: true,
    captureFrontendLogs: true,
  }]],
  framework: "mocha",
  reporters: artifactDirectory
    ? ["spec", ["junit", {
      outputDir: artifactDirectory,
      outputFileFormat: () => "wdio-results.xml",
    }]]
    : ["spec"],
  logLevel: nativeSmoke ? "info" : "warn",
  bail: 1,
  waitforTimeout: 15_000,
  connectionRetryTimeout: 90_000,
  // A timed-out mutation must not start a second overlapping activation.
  connectionRetryCount: activationDiagnostic ? 0 : 1,
  mochaOpts: { ui: "bdd", timeout: activationDiagnostic ? EXTENSION_DIAGNOSTIC_TIMEOUT_MS : mochaTimeoutMs },
};
