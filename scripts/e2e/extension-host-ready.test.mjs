import assert from "node:assert/strict";
import test from "node:test";
import { tsImport } from "tsx/esm/api";

const { waitForHostReady } = await tsImport("./extension-host-ready.ts", import.meta.url);

// Like WDIO, this bounded poller retries rejected predicates. A terminal state
// must complete the predicate, with failure propagated outside the poller.
async function poll(condition) {
  for (let attempt = 0; attempt < 3; attempt += 1) {
    try {
      if (await condition()) return;
    } catch {
      continue;
    }
  }
  throw new Error("poll timeout");
}

test("terminal host errors fail after one observation, preserving a safe code", async () => {
  let reads = 0;
  await assert.rejects(waitForHostReady(async () => {
    reads += 1;
    return { state: "error", lastError: "extensions_host_unavailable" };
  }, poll), /extensions_host_unavailable/u);
  assert.equal(reads, 1);
});

test("startup waits for running and timeout remains a failure", async () => {
  let reads = 0;
  await waitForHostReady(async () => ({ state: ++reads === 2 ? "running" : "starting" }), poll);
  assert.equal(reads, 2);
  await assert.rejects(waitForHostReady(async () => ({ state: "starting" }), poll), /poll timeout/u);
});

test("unexpected host details are not exposed in the error", async () => {
  await assert.rejects(waitForHostReady(async () => ({
    state: "error", lastError: "private path or token",
  }), poll), { message: "Extension host unavailable: host_error" });
});
