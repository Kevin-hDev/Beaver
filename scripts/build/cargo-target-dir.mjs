import { lstat, realpath } from "node:fs/promises";
import { isAbsolute, normalize, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const ERROR_MESSAGE = "Cargo target validation failed";
const MAX_PATH_LENGTH = 30_000;
const CONTROL_PATTERN = /[\u0000-\u001f\u007f-\u009f]/u;

function fail() {
  throw new Error(ERROR_MESSAGE);
}

function comparablePath(value) {
  const comparable = value
    .replace(/^\\\\\?\\UNC\\/iu, "\\\\")
    .replace(/^\\\\\?\\/u, "")
    .replaceAll("/", "\\");
  return process.platform === "win32" ? comparable.toLowerCase() : value;
}

function hasTraversal(value) {
  return value.split(/[\\/]+/u).includes("..");
}

export function normalizeCargoTargetDir(value) {
  try {
    if (
      typeof value !== "string" ||
      value.length < 1 ||
      value.length > MAX_PATH_LENGTH ||
      CONTROL_PATTERN.test(value) ||
      hasTraversal(value) ||
      !isAbsolute(value)
    ) {
      fail();
    }
    const normalized = normalize(value);
    if (
      normalized.length < 1 ||
      normalized.length > MAX_PATH_LENGTH ||
      CONTROL_PATTERN.test(normalized) ||
      hasTraversal(normalized) ||
      !isAbsolute(normalized)
    ) {
      fail();
    }
    return normalized;
  } catch {
    fail();
  }
}

export async function canonicalCargoTargetDir(value) {
  try {
    const normalized = normalizeCargoTargetDir(value);
    const [info, canonical] = await Promise.all([
      lstat(normalized),
      realpath(normalized),
    ]);
    if (
      !info.isDirectory() ||
      info.isSymbolicLink() ||
      comparablePath(normalized) !== comparablePath(canonical)
    ) {
      fail();
    }
    return canonical;
  } catch {
    fail();
  }
}

async function main() {
  if (process.argv.length !== 2) fail();
  const target = await canonicalCargoTargetDir(process.env.CARGO_TARGET_DIR);
  process.stdout.write(`${target}\n`);
}

const invokedPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invokedPath) {
  main().catch(() => {
    process.stderr.write(`${ERROR_MESSAGE}\n`);
    process.exitCode = 1;
  });
}
