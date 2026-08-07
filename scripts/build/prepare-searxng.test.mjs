import assert from "node:assert/strict";
import { mkdtemp, mkdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";

import { prepareSearxng } from "./prepare-searxng.mjs";

async function makeRepository() {
  const root = await mkdtemp(join(await realpath(tmpdir()), "searxng-bridge-"));
  await mkdir(join(root, "src-tauri", "scripts"), { recursive: true });
  await writeFile(join(root, "src-tauri", "scripts", "prepare_searxng.py"), "pass\n");
  return root;
}

test("runs the canonical Python preparation script with separated arguments", async () => {
  const repoRoot = await makeRepository();
  const calls = [];
  try {
    await prepareSearxng({
      repoRoot,
      resolvePython: async () => ({ command: "py", prefixArgs: ["-3"] }),
      run: async (call) => calls.push(call),
    });
    assert.equal(calls.length, 1);
    assert.equal(calls[0].command, "py");
    assert.deepEqual(calls[0].args, [
      "-3",
      resolve(repoRoot, "src-tauri/scripts/prepare_searxng.py"),
      "--root",
      resolve(repoRoot, "src-tauri"),
    ]);
    assert.equal(calls[0].cwd, await realpath(repoRoot));
  } finally {
    await rm(repoRoot, { recursive: true, force: true });
  }
});

test("refuses traversal roots and malformed Python candidates without launching", async () => {
  const repoRoot = await makeRepository();
  let launched = false;
  try {
    await assert.rejects(() => prepareSearxng({
      repoRoot: join(repoRoot, "..", "repository"),
      resolvePython: async () => ({ command: "python", prefixArgs: [] }),
      run: async () => { launched = true; },
    }), (error) => error.message === "SearXNG preparation failed");
    await assert.rejects(() => prepareSearxng({
      repoRoot,
      resolvePython: async () => ({ command: "python\n", prefixArgs: [] }),
      run: async () => { launched = true; },
    }), (error) => error.message === "SearXNG preparation failed");
    assert.equal(launched, false);
  } finally {
    await rm(repoRoot, { recursive: true, force: true });
  }
});
