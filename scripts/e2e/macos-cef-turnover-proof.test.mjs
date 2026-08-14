import assert from "node:assert/strict";
import { mkdtemp, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  captureMacCefTurnoverProof,
  waitForMacCefTurnoverProof,
} from "./macos-cef-turnover-proof.mjs";

const MAC_ROOT = "/build/target/e2e/debug/bundle/macos/Beaver.app";

test("the pre-launch observer publishes one atomic turnover proof", async (context) => {
  const logDirectory = await temporaryDirectory(context);
  await captureMacCefTurnoverProof({
    logDirectory,
    root: MAC_ROOT,
    timeoutMs: 50,
    observeTurnover: async () => ({ exitedPid: 42, initialPids: [42, 43] }),
  });

  const proof = await waitForMacCefTurnoverProof({
    logDirectory,
    timeoutMs: 50,
    pollMs: 1,
  });
  assert.equal(proof.exitedPid, 42);
  assert.deepEqual(proof.initialPids, [42, 43]);
  assert.deepEqual(await readdir(logDirectory), ["cef-helper-turnover.json"]);
});

test("an observer failure is published and fails the native proof", async (context) => {
  const logDirectory = await temporaryDirectory(context);
  await captureMacCefTurnoverProof({
    logDirectory,
    root: MAC_ROOT,
    timeoutMs: 50,
    observeTurnover: async () => {
      throw new Error("injected observer failure");
    },
  });

  await assert.rejects(
    waitForMacCefTurnoverProof({ logDirectory, timeoutMs: 50, pollMs: 1 }),
    /Native CEF turnover proof failed/u,
  );
});

async function temporaryDirectory(context) {
  const directory = await mkdtemp(join(tmpdir(), "beaver-cef-turnover-"));
  context.after(() => rm(directory, { recursive: true, force: true }));
  return directory;
}
