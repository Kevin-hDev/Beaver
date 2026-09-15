import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { observeExit, writeClosureReport } from "./run-native-probe.mjs";

test("observes only the dedicated child process", async () => {
  const root = await mkdtemp(join(tmpdir(), "beaver-voice-probe-observer-"));
  const evidencePath = join(root, "voice-probe-cooperative.marker");
  const result = await observeExit(process.execPath, "cooperative", {
    cwd: process.cwd(),
    evidencePath,
    spawnProcess: (binary, _args, options) =>
      spawn(binary, ["-e", "require('fs').writeFileSync(process.env.VOICE_PROBE_EXIT_EVIDENCE, 'cooperative-released\\n'); setTimeout(() => process.exit(0), 450)"], options),
  });
  assert.equal(result.exited, true);
  assert.equal(result.cleanupConfirmed, true);
  assert.equal(result.signal, null);
});

test("rejects an instance forwarded to another Beaver process", async () => {
  const result = await observeExit(process.execPath, "forced", {
    cwd: process.cwd(),
    spawnProcess: (binary, _args, options) => spawn(binary, ["-e", ""], options),
  });
  assert.equal(result.exited, false);
  assert.equal(result.cleanupConfirmed, false);
});

test("writes separate cooperative and forced evidence", async () => {
  const root = await mkdtemp(join(tmpdir(), "beaver-voice-probe-"));
  const path = join(root, "closure.json");
  await writeClosureReport(path, [
    { mode: "cooperative", exited: true, cleanupConfirmed: true },
    { mode: "forced", exited: true, cleanupConfirmed: false },
  ]);
  const report = JSON.parse(await readFile(path, "utf8"));
  assert.equal(report.reports[1].cleanupConfirmed, false);
});
