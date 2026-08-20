import assert from "node:assert/strict";
import { mkdtemp, mkdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";

import { prepareSearxng } from "./prepare-searxng.mjs";

async function makeRepository() {
  const root = await mkdtemp(join(await realpath(tmpdir()), "searxng-bridge-"));
  await mkdir(join(root, "src-tauri", "scripts"), { recursive: true });
  await mkdir(join(root, "scripts", "build"), { recursive: true });
  await writeFile(join(root, "src-tauri", "scripts", "prepare_searxng.py"), "pass\n");
  await writeFile(join(root, "scripts", "build", "searxng-python-version.txt"), "3.14\n");
  return root;
}

test("runs the canonical Python preparation script with separated arguments", async () => {
  const repoRoot = await makeRepository();
  const calls = [];
  const resolverCalls = [];
  try {
    await prepareSearxng({
      repoRoot,
      resolvePython: async (request) => {
        resolverCalls.push(request);
        return { command: "py", prefixArgs: ["-3.14"], label: "py-3.14" };
      },
      run: async (call) => calls.push(call),
    });
    assert.equal(calls.length, 1);
    assert.equal(calls[0].command, "py");
    assert.deepEqual(calls[0].args, [
      "-3.14",
      resolve(repoRoot, "src-tauri/scripts/prepare_searxng.py"),
      "--root",
      resolve(repoRoot, "src-tauri"),
    ]);
    assert.equal(calls[0].cwd, await realpath(repoRoot));
    assert.deepEqual(resolverCalls, [{
      platform: process.platform,
      expectedVersion: { major: 3, minor: 14, label: "3.14" },
    }]);
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
      resolvePython: async () => ({ command: "python", prefixArgs: [], label: "python" }),
      run: async () => { launched = true; },
    }), (error) => error.message === "SearXNG preparation failed");
    await assert.rejects(() => prepareSearxng({
      repoRoot,
      resolvePython: async () => ({ command: "python\n", prefixArgs: [], label: "python" }),
      run: async () => { launched = true; },
    }), (error) => error.message === "SearXNG preparation failed");
    assert.equal(launched, false);
  } finally {
    await rm(repoRoot, { recursive: true, force: true });
  }
});
