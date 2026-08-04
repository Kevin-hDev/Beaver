import { lstat, realpath } from "node:fs/promises";
import { isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { runCommand } from "./command-runner.mjs";
import { canonicalDirectory } from "./updater-helper-copy.mjs";

const ERROR_MESSAGE = "Frontend build failed";
const MAX_PATH_LENGTH = 4096;

function fail() {
  throw new Error(ERROR_MESSAGE);
}

function isInside(root, path) {
  const child = relative(root, path);
  return child.length > 0 && !child.startsWith("..") && !isAbsolute(child);
}

async function canonicalFile(root, segments) {
  try {
    const expected = join(root, ...segments);
    if (expected.length > MAX_PATH_LENGTH || !isInside(root, expected)) fail();
    const [info, canonical] = await Promise.all([lstat(expected), realpath(expected)]);
    if (!info.isFile() || info.isSymbolicLink() || canonical !== expected) fail();
    return canonical;
  } catch {
    fail();
  }
}

export async function buildFrontend({ repoRoot, run = runCommand } = {}) {
  try {
    if (typeof run !== "function") fail();
    const root = await canonicalDirectory(repoRoot);
    const checks = await canonicalFile(root, ["scripts", "check-react-component-calls.mjs"]);
    const typescript = await canonicalFile(root, ["node_modules", "typescript", "bin", "tsc"]);
    const vite = await canonicalFile(root, ["node_modules", "vite", "bin", "vite.js"]);
    await run({ command: process.execPath, args: [checks], cwd: root });
    await run({ command: process.execPath, args: [typescript], cwd: root });
    await run({ command: process.execPath, args: [vite, "build"], cwd: root });
  } catch {
    fail();
  }
}

const invokedPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invokedPath) {
  const repoRoot = resolve(fileURLToPath(new URL("../..", import.meta.url)));
  buildFrontend({ repoRoot }).catch(() => {
    process.stderr.write(`${ERROR_MESSAGE}\n`);
    process.exitCode = 1;
  });
}
