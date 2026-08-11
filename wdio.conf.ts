const appBinaryPath = process.env.E2E_APP_BINARY;
if (!appBinaryPath) throw new Error("E2E app binary is not configured");
const e2eLogDirectory = process.env.E2E_LOG_DIR;
if (!e2eLogDirectory) throw new Error("E2E log directory is not configured");
const nativeCefSmoke = process.env.E2E_REQUIRE_CEF_SMOKE === "1";

export const config: WebdriverIO.Config = {
  outputDir: e2eLogDirectory,
  runner: "local",
  specs: [nativeCefSmoke
    ? "./tests/e2e/native-cef-shutdown.spec.ts"
    : "./tests/e2e/onboarding.spec.ts"],
  maxInstances: 1,
  capabilities: [{ browserName: "tauri" }],
  services: [["@wdio/tauri-service", {
    appBinaryPath,
    driverProvider: "embedded",
    captureBackendLogs: true,
    captureFrontendLogs: true,
  }]],
  framework: "mocha",
  reporters: ["spec"],
  logLevel: nativeCefSmoke ? "info" : "warn",
  bail: 1,
  waitforTimeout: 15_000,
  connectionRetryTimeout: 90_000,
  connectionRetryCount: 1,
  mochaOpts: { ui: "bdd", timeout: 60_000 },
};
