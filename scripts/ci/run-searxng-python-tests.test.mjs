import assert from "node:assert/strict";
import { mkdtemp, mkdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { runSearxngPythonTests } from "./run-searxng-python-tests.mjs";

test("lance unittest avec le CPython contrôlé et des arguments séparés", async () => {
  const root = await mkdtemp(join(await realpath(tmpdir()), "searxng-python-tests-"));
  const calls = [];
  try {
    await mkdir(join(root, "scripts", "build"), { recursive: true });
    await mkdir(join(root, "src-tauri", "scripts"), { recursive: true });
    await writeFile(join(root, "scripts", "build", "searxng-python-version.txt"), "3.14\n");
    await runSearxngPythonTests({
      repoRoot: root,
      resolvePython: async () => ({ command: "py", prefixArgs: ["-3.14"], label: "py-3.14" }),
      run: async (call) => calls.push(call),
    });

    assert.deepEqual(calls, [{
      command: "py",
      args: [
        "-3.14",
        "-m",
        "unittest",
        "discover",
        "-s",
        "src-tauri/scripts",
        "-p",
        "test_*.py",
      ],
      cwd: await realpath(root),
      timeoutMs: 120_000,
    }]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("la résolution par défaut lit la version contrôlée avant de lancer les tests", async () => {
  const root = await mkdtemp(join(await realpath(tmpdir()), "searxng-python-authority-"));
  const observed = [];
  try {
    await mkdir(join(root, "scripts", "build"), { recursive: true });
    await mkdir(join(root, "src-tauri", "scripts"), { recursive: true });
    await writeFile(join(root, "scripts", "build", "searxng-python-version.txt"), "3.99\n");
    await runSearxngPythonTests({
      repoRoot: root,
      probePython: async (candidate, expectedVersion) => {
        observed.push({ candidate, expectedVersion });
        return true;
      },
      run: async () => {},
    });

    assert.equal(observed.length, 1);
    assert.equal(observed[0].candidate.label, "python3.99");
    assert.deepEqual(observed[0].expectedVersion, { major: 3, minor: 99, label: "3.99" });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
