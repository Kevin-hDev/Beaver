import assert from "node:assert/strict";
import { test } from "node:test";
import {
  exitDiagnostic,
  isAllowedObservedBinary,
  observedLaunch,
} from "./macos-app-observer.mjs";

test("the macOS observer accepts only one canonical absolute application binary", () => {
  assert.equal(isAllowedObservedBinary("/Applications/Beaver.app/Contents/MacOS/cl-go-dash"), true);
  assert.equal(isAllowedObservedBinary("relative/cl-go-dash"), false);
  assert.equal(isAllowedObservedBinary("/Applications/../tmp/cl-go-dash"), false);
  assert.equal(isAllowedObservedBinary("/Applications/Beaver.app\nsecret"), false);
});

test("the macOS observer launches the application without a shell or inherited arguments", () => {
  assert.deepEqual(observedLaunch("/Applications/Beaver.app/Contents/MacOS/cl-go-dash"), {
    command: "/Applications/Beaver.app/Contents/MacOS/cl-go-dash",
    args: [],
    options: {
      env: process.env,
      shell: false,
      stdio: "inherit",
    },
  });
});

test("the macOS observer reports only bounded exit categories", () => {
  assert.equal(exitDiagnostic(0, null), "[e2e-process] application-exit-code-0");
  assert.equal(exitDiagnostic(101, null), "[e2e-process] application-exit-code-101");
  assert.equal(exitDiagnostic(null, "SIGABRT"), "[e2e-process] application-exit-signal-sigabrt");
  assert.equal(exitDiagnostic(null, "UNTRUSTED"), "[e2e-process] application-exit-signal-unknown");
});
