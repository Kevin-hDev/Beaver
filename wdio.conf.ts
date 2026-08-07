const appBinaryPath = process.env.E2E_APP_BINARY;
if (!appBinaryPath) throw new Error("E2E app binary is not configured");

export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./tests/e2e/**/*.spec.ts"],
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
  logLevel: "warn",
  bail: 1,
  waitforTimeout: 15_000,
  connectionRetryTimeout: 90_000,
  connectionRetryCount: 1,
  mochaOpts: { ui: "bdd", timeout: 60_000 },
};
