import assert from "node:assert/strict";
import test from "node:test";
import { tsImport } from "tsx/esm/api";

import {
  NATIVE_JOURNEY_MOCHA_TIMEOUT_MS,
} from "./native-journey-deadline.mjs";

test("WebdriverIO covers both native journeys and extension setup", async () => {
  const previousBinary = process.env.E2E_APP_BINARY;
  const previousLogDirectory = process.env.E2E_LOG_DIR;
  const previousArtifactDirectory = process.env.E2E_ARTIFACT_DIR;
  process.env.E2E_APP_BINARY = "C:\\bounded-e2e\\beaver.exe";
  process.env.E2E_LOG_DIR = "C:\\bounded-e2e\\logs";
  process.env.E2E_ARTIFACT_DIR = "C:\\bounded-e2e\\artifacts";
  try {
    const [module, deadlines] = await Promise.all([
      tsImport("../../wdio.conf.ts", import.meta.url),
      tsImport("./extension-setup-deadline.ts", import.meta.url),
    ]);

    assert.ok(
      deadlines.EXTENSION_UI_SETUP_TIMEOUT_MS > NATIVE_JOURNEY_MOCHA_TIMEOUT_MS,
    );
    assert.equal(
      module.config.mochaOpts?.timeout,
      Math.max(
        NATIVE_JOURNEY_MOCHA_TIMEOUT_MS,
        deadlines.EXTENSION_UI_SETUP_TIMEOUT_MS,
      ),
    );
    assert.equal(module.config.reporters?.[0], "spec");
    assert.equal(module.config.reporters?.[1]?.[0], "junit");
    assert.equal(
      module.config.reporters?.[1]?.[1]?.outputDir,
      "C:\\bounded-e2e\\artifacts",
    );
    assert.equal(
      module.config.reporters?.[1]?.[1]?.outputFileFormat(),
      "wdio-results.xml",
    );
  } finally {
    restoreEnvironment("E2E_APP_BINARY", previousBinary);
    restoreEnvironment("E2E_LOG_DIR", previousLogDirectory);
    restoreEnvironment("E2E_ARTIFACT_DIR", previousArtifactDirectory);
  }
});

function restoreEnvironment(name, value) {
  if (value === undefined) delete process.env[name];
  else process.env[name] = value;
}
