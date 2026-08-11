import assert from "node:assert/strict";
import { test } from "node:test";
import {
  createDiagnosticBuffer,
  diagnosticFilePath,
  exitDiagnostic,
  safeObservedDiagnostic,
} from "./macos-app-diagnostics.mjs";
import {
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
      stdio: ["ignore", "inherit", "pipe"],
    },
  });
});

test("the macOS observer reports only bounded exit categories", () => {
  assert.equal(exitDiagnostic(0, null), "[e2e-process] application-exit-code-0");
  assert.equal(exitDiagnostic(101, null), "[e2e-process] application-exit-code-101");
  assert.equal(exitDiagnostic(null, "SIGABRT"), "[e2e-process] application-exit-signal-sigabrt");
  assert.equal(exitDiagnostic(null, "UNTRUSTED"), "[e2e-process] application-exit-signal-unknown");
});

test("the macOS observer persists only inside its canonical temporary E2E profile", () => {
  const identity = (value) => value;
  assert.equal(
    diagnosticFilePath("/private/tmp/beaver-e2e-Ab12/logs", {
      realpath: identity,
      temporaryDirectory: "/private/tmp",
    }),
    "/private/tmp/beaver-e2e-Ab12/logs/native-app-exit.log",
  );
  assert.equal(diagnosticFilePath("/private/tmp/another-profile/logs", {
    realpath: identity,
    temporaryDirectory: "/private/tmp",
  }), undefined);
  assert.equal(diagnosticFilePath("/private/tmp/beaver-e2e-Ab12/../logs", {
    realpath: identity,
    temporaryDirectory: "/private/tmp",
  }), undefined);
});

test("the macOS observer accepts only fixed lifecycle and run-event markers", () => {
  assert.equal(
    safeObservedDiagnostic("[e2e-lifecycle] setup-completed"),
    "[e2e-lifecycle] setup-completed",
  );
  assert.equal(
    safeObservedDiagnostic("[e2e-run-event] exit-requested-user"),
    "[e2e-run-event] exit-requested-user",
  );
  assert.equal(
    safeObservedDiagnostic("[e2e-exit-source] browser-child-admission"),
    "[e2e-exit-source] browser-child-admission",
  );
  assert.equal(safeObservedDiagnostic("[e2e-run-event] secret=/private/path"), undefined);
  assert.equal(safeObservedDiagnostic("[e2e-lifecycle] setup-completed extra"), undefined);
});

test("the macOS observer reconstructs split markers with bounded line storage", () => {
  const diagnostics = [];
  const capture = createDiagnosticBuffer((value) => diagnostics.push(value));
  capture.push(Buffer.from("[e2e-life", "utf8"));
  capture.push(Buffer.from("cycle] event-loop-entered\n", "utf8"));
  capture.push(Buffer.alloc(512, 65));
  capture.push(Buffer.from("\n[e2e-run-event] exit\n", "utf8"));
  capture.finish();
  assert.deepEqual(diagnostics, [
    "[e2e-lifecycle] event-loop-entered",
    "[e2e-run-event] exit",
  ]);
});
