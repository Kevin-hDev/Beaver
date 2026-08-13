import assert from "node:assert/strict";
import test from "node:test";
import { tsImport } from "tsx/esm/api";

import {
  NATIVE_JOURNEY_MOCHA_TIMEOUT_MS,
} from "./native-journey-deadline.mjs";

test("WebdriverIO derives its Mocha guard from the native journey policy", async () => {
  const previousBinary = process.env.E2E_APP_BINARY;
  const previousLogDirectory = process.env.E2E_LOG_DIR;
  process.env.E2E_APP_BINARY = "C:\\bounded-e2e\\beaver.exe";
  process.env.E2E_LOG_DIR = "C:\\bounded-e2e\\logs";
  try {
    const module = await tsImport("../../wdio.conf.ts", import.meta.url);

    assert.equal(module.config.mochaOpts?.timeout, NATIVE_JOURNEY_MOCHA_TIMEOUT_MS);
  } finally {
    restoreEnvironment("E2E_APP_BINARY", previousBinary);
    restoreEnvironment("E2E_LOG_DIR", previousLogDirectory);
  }
});

function restoreEnvironment(name, value) {
  if (value === undefined) delete process.env[name];
  else process.env[name] = value;
}
