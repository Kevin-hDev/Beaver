import assert from "node:assert/strict";
import test from "node:test";
import { markerSummary, safeOutcome, verifiedScriptTimeout } from "./activation-diagnostic.mjs";

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
