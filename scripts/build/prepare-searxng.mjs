import { lstatSync, realpathSync } from "node:fs";
import { isAbsolute, join, parse, relative, sep } from "node:path";

import { runCommand } from "./command-runner.mjs";
import { resolvePythonCommand } from "./python-runtime.mjs";

const ERROR_MESSAGE = "SearXNG preparation failed";
const MAX_PATH_LENGTH = 4096;
const MAX_COMMAND_LENGTH = 512;
const MAX_PREFIX_ARGUMENTS = 4;

function fail() {
  throw new Error(ERROR_MESSAGE);
}

function comparablePath(value) {
  if (process.platform !== "win32") return value;
  return value
    .replace(/^\\\\\?\\UNC\\/iu, "\\\\")
    .replace(/^\\\\\?\\/u, "")
    .replaceAll("/", "\\")
    .toLowerCase();
}

function hasTraversal(value) {
  return value.split(/[\\/]+/u).includes("..");
}

function hasLink(path) {
  const root = parse(path).root;
  let current = root;
  for (const segment of relative(root, path).split(sep).filter(Boolean)) {
    current = join(current, segment);
    if (lstatSync(current).isSymbolicLink()) return true;
  }
  return false;
}

function canonicalDirectory(value) {
  if (typeof value !== "string" || value.length === 0 || value.length > MAX_PATH_LENGTH || /[\0\r\n]/u.test(value) || !isAbsolute(value) || hasTraversal(value)) {
    fail();
  }
  try {
    const canonical = realpathSync.native(value);
    if (!lstatSync(canonical).isDirectory() || comparablePath(value) !== comparablePath(canonical) || hasLink(value)) fail();
    return canonical;
  } catch {
    fail();
  }
}

function validCandidate(candidate) {
  return (
    typeof candidate === "object" &&
    candidate !== null &&
    typeof candidate.command === "string" &&
    candidate.command.length > 0 &&
    candidate.command.length <= MAX_COMMAND_LENGTH &&
    !/[\0\r\n]/u.test(candidate.command) &&
    Array.isArray(candidate.prefixArgs) &&
    candidate.prefixArgs.length <= MAX_PREFIX_ARGUMENTS &&
    candidate.prefixArgs.every((argument) => typeof argument === "string" && argument.length <= MAX_COMMAND_LENGTH && !/[\0\r\n]/u.test(argument))
  );
}

export async function prepareSearxng({ repoRoot, resolvePython = resolvePythonCommand, run = runCommand } = {}) {
  try {
    if (typeof resolvePython !== "function" || typeof run !== "function") fail();
    const root = canonicalDirectory(repoRoot);
    const tauriRoot = canonicalDirectory(join(root, "src-tauri"));
    const script = join(tauriRoot, "scripts", "prepare_searxng.py");
    const canonicalScript = realpathSync.native(script);
    if (!lstatSync(canonicalScript).isFile() || comparablePath(script) !== comparablePath(canonicalScript) || hasLink(script)) fail();
    const candidate = await resolvePython({ platform: process.platform });
    if (!validCandidate(candidate)) fail();
    const command = candidate.command;
    const prefixArgs = [...candidate.prefixArgs];
    if (!validCandidate({ command, prefixArgs })) fail();
    await run({
      command,
      args: [...prefixArgs, canonicalScript, "--root", tauriRoot],
      cwd: root,
      timeoutMs: 300000,
    });
  } catch {
    fail();
  }
}
