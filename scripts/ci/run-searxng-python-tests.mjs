import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { runCommand } from "../build/command-runner.mjs";
import { readSupportedPythonVersion, resolvePythonCommand } from "../build/python-runtime.mjs";
import { canonicalDirectory } from "../build/updater-helper-copy.mjs";

const ERROR_MESSAGE = "SearXNG script tests failed";

export async function runSearxngPythonTests({
  repoRoot,
  resolvePython = resolvePythonCommand,
  run = runCommand,
} = {}) {
  try {
    if (typeof resolvePython !== "function" || typeof run !== "function") throw new Error();
    const root = await canonicalDirectory(repoRoot);
    const expectedVersion = readSupportedPythonVersion(root);
    const candidate = await resolvePython({ platform: process.platform, expectedVersion });
    await run({
      command: candidate.command,
      args: [
        ...candidate.prefixArgs,
        "-m",
        "unittest",
        "discover",
        "-s",
        "src-tauri/scripts",
        "-p",
        "test_*searxng*.py",
      ],
      cwd: root,
      timeoutMs: 120_000,
    });
  } catch {
    throw new Error(ERROR_MESSAGE);
  }
}

const invokedPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invokedPath) {
  const repoRoot = resolve(fileURLToPath(new URL("../..", import.meta.url)));
  runSearxngPythonTests({ repoRoot }).catch(() => {
    process.stderr.write(`${ERROR_MESSAGE}\n`);
    process.exitCode = 1;
  });
}
