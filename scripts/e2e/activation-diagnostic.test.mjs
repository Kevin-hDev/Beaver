import assert from "node:assert/strict";
import test from "node:test";
import { mkdir, mkdtemp, realpath, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { diagnosticProfile, markerSummary, safeOutcome, verifiedScriptTimeout } from "./activation-diagnostic.mjs";

test("marker sampling accepts isolated data and rejects unrelated directories", async () => {
  const profile = await mkdtemp(join(tmpdir(), "beaver-e2e-"));
  try {
    await mkdir(join(profile, "data"));
    await mkdir(join(profile, "other"));
    assert.equal(await diagnosticProfile(profile), await realpath(profile));
    assert.equal(await diagnosticProfile(join(profile, "data")), await realpath(join(profile, "data")));
    await assert.rejects(diagnosticProfile(join(profile, "other")), /unavailable/u);
    await assert.rejects(diagnosticProfile(tmpdir()), /unavailable/u);
  } finally {
    await rm(profile, { recursive: true, force: true });
  }
});

test("diagnostics retain stages but never arbitrary marker data", () => {
  assert.deepEqual(markerSummary(Buffer.from(JSON.stringify({version: 2, host: {
    extensionId: "acceptance.standard.complete", stage: "activate", secret: "private",
  }}))), {state: "present", extensionId: "acceptance.standard.complete", stage: "activate"});
  assert.deepEqual(markerSummary(Buffer.from('{"host":{"extensionId":"secret/evil"}}')), {state: "invalid"});
  assert.deepEqual(markerSummary(Buffer.alloc(2049)), {state: "oversized"});
  assert.equal(safeOutcome(new Error("private path token")), "operation-failed");
});

test("timeouts are applied and read back, not assumed from capabilities", async () => {
  let script = 30000;
  const driver = {
    getTimeouts: async () => ({script}),
    setTimeout: async (value) => { script = value.script; },
  };
  assert.deepEqual(await verifiedScriptTimeout(driver, 95000), {previous: 30000, effective: 95000});
  driver.setTimeout = async () => {};
  await assert.rejects(verifiedScriptTimeout(driver, 96000), /not applied/u);
});
