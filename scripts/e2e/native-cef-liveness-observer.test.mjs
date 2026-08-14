import assert from "node:assert/strict";
import test from "node:test";

import {
  observeOwnedCefHelperTurnover,
  waitForOwnedCefHelperSet,
  waitForOwnedCefHelperTurnover,
} from "./native-cef-liveness-observer.mjs";

const MAC_ROOT = "/build/target/e2e/debug/bundle/macos/Beaver.app";

test("helper capture returns sorted unique owned pids", async () => {
  const result = await waitForOwnedCefHelperSet({
    platform: "darwin",
    root: MAC_ROOT,
    timeoutMs: 50,
    pollMs: 1,
    listProcesses: () => [helper(43), helper(42), helper(43)],
  });

  assert.deepEqual(result, [42, 43]);
});

test("turnover resolves only after an initially owned helper disappears", async () => {
  let polls = 0;
  const result = await waitForOwnedCefHelperTurnover({
    platform: "darwin",
    root: MAC_ROOT,
    initialPids: [42, 43],
    timeoutMs: 50,
    pollMs: 1,
    listProcesses: () => (polls++ === 0 ? [helper(42), helper(43)] : [helper(43)]),
  });

  assert.equal(result.exitedPid, 42);
  assert.deepEqual(result.initialPids, [42, 43]);
  assert.equal(polls, 2);
});

test("turnover tracks a later owned helper and resolves when it disappears", async () => {
  let polls = 0;
  const snapshots = [
    [helper(42)],
    [helper(42), helper(44)],
    [helper(42)],
  ];
  const result = await waitForOwnedCefHelperTurnover({
    platform: "darwin",
    root: MAC_ROOT,
    initialPids: [42],
    timeoutMs: 50,
    pollMs: 1,
    listProcesses: () => snapshots[Math.min(polls++, snapshots.length - 1)],
  });

  assert.equal(result.exitedPid, 44);
  assert.deepEqual(result.initialPids, [42]);
  assert.equal(polls, 3);
});

test("rolling observation catches a helper that exits before browser capability is ready", async () => {
  let polls = 0;
  const snapshots = [
    [],
    [helper(42), helper(43)],
    [helper(43)],
  ];
  const result = await observeOwnedCefHelperTurnover({
    platform: "darwin",
    root: MAC_ROOT,
    timeoutMs: 50,
    pollMs: 1,
    listProcesses: () => snapshots[Math.min(polls++, snapshots.length - 1)],
  });

  assert.equal(result.exitedPid, 42);
  assert.deepEqual(result.initialPids, [42, 43]);
  assert.equal(polls, 3);
});

test("helper capture without an owned helper fails closed", async () => {
  await assert.rejects(
    waitForOwnedCefHelperSet({
      platform: "darwin",
      root: MAC_ROOT,
      timeoutMs: 2,
      pollMs: 1,
      listProcesses: () => [],
    }),
    /Native CEF liveness observation failed/u,
  );
});

test("a helper that never exits reaches the bounded timeout", async () => {
  await assert.rejects(
    waitForOwnedCefHelperTurnover({
      platform: "darwin",
      root: MAC_ROOT,
      initialPids: [42],
      timeoutMs: 2,
      pollMs: 1,
      listProcesses: () => [helper(42)],
    }),
    /Native CEF liveness observation failed/u,
  );
});

test("more than 64 initial helper pids fails closed", async () => {
  await assert.rejects(
    waitForOwnedCefHelperTurnover({
      platform: "darwin",
      root: MAC_ROOT,
      initialPids: Array.from({ length: 65 }, (_, index) => index + 2),
      timeoutMs: 50,
      pollMs: 1,
      listProcesses: () => [],
    }),
    /Native CEF liveness observation failed/u,
  );
});

test("a sixty-fifth cumulatively observed helper fails closed", async () => {
  let polls = 0;
  const initialPids = Array.from({ length: 64 }, (_, index) => index + 2);
  await assert.rejects(
    waitForOwnedCefHelperTurnover({
      platform: "darwin",
      root: MAC_ROOT,
      initialPids,
      timeoutMs: 50,
      pollMs: 1,
      listProcesses: () => {
        polls += 1;
        return polls === 1
          ? initialPids.map(helper)
          : [...initialPids.map(helper), helper(66)];
      },
    }),
    /Native CEF liveness observation failed/u,
  );
  assert.equal(polls, 2);
});

function helper(pid) {
  return {
    pid,
    parentPid: 1,
    executable: "",
    command: `${MAC_ROOT}/Contents/Frameworks/Beaver Helper.app/Contents/MacOS/Beaver Helper --type=renderer`,
  };
}
