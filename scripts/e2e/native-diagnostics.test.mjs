import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  collectNativeCefDiagnostics,
  MAX_DIAGNOSTIC_FILES,
} from "./native-diagnostics.mjs";

test("native CEF diagnostics expose only bounded browser failure categories", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-native-logs-"));
  try {
    await writeFile(
      join(directory, "wdio-2026-08-11T05-38-25-724Z.log"),
      [
        "secret=/private/path token=do-not-print",
        "[Tauri:Backend] [browser] macOS supervision failed (cef-supervision-admission)",
        "[Tauri:Backend] [browser] initialization failed after CEF boundary",
      ].join("\n"),
      "utf8",
    );

    assert.deepEqual(await collectNativeCefDiagnostics(directory), [
      "browser-supervision:cef-supervision-admission",
      "browser-initialization:fatal",
    ]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("native CEF diagnostics expose only fixed lifecycle stages", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-native-logs-"));
  try {
    await writeFile(
      join(directory, "wdio-lifecycle.log"),
      [
        "Tauri app spawned (PID: 1234) secret=do-not-print",
        "WebDriver server ready on port 4445",
        "Embedded WebDriver on port 4445 (instance: 0) is healthy",
        "[Tauri:Backend] [exit] coordinated shutdown requested",
        "[Tauri:Backend] [exit] event loop returned",
        "[Tauri:Backend] [browser] launch callback failed",
        "[browser-helper] setup failed (cef-supervision-admission)",
        "[e2e-lifecycle] setup-completed",
        "[e2e-lifecycle] event-loop-entered",
        "[e2e-process] application-exit-signal-sigsegv",
      ].join("\n"),
      "utf8",
    );

    assert.deepEqual(await collectNativeCefDiagnostics(directory), [
      "webdriver:spawned",
      "webdriver:ready",
      "webdriver:healthy",
      "application-exit:coordinated",
      "application-exit:event-loop",
      "browser-callback:fatal",
      "browser-helper:cef-supervision-admission",
      "application-stage:setup-completed",
      "application-stage:event-loop-entered",
      "process-exit:signal-sigsegv",
    ]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("native CEF diagnostics bound files and ignore unrelated names", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-native-logs-"));
  try {
    await writeFile(
      join(directory, "untrusted.log"),
      "[browser] initialization failed after CEF boundary",
      "utf8",
    );
    for (let index = 0; index < MAX_DIAGNOSTIC_FILES + 3; index += 1) {
      await writeFile(
        join(directory, `wdio-${String(index).padStart(2, "0")}.log`),
        `[browser] preflight unavailable (cef-supervision-object)\n`,
        "utf8",
      );
    }

    assert.deepEqual(await collectNativeCefDiagnostics(directory), [
      "browser-preflight:cef-supervision-object",
    ]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("a missing diagnostic directory is a safe empty result", async () => {
  const missing = join(tmpdir(), "beaver-native-logs-missing");
  await rm(missing, { recursive: true, force: true });
  assert.deepEqual(await collectNativeCefDiagnostics(missing), []);
});
