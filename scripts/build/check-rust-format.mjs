import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { resolve } from "node:path";

import { runCommand } from "./command-runner.mjs";

const MAX_CHANGED_FILES = 4_096;
const MAX_GIT_OUTPUT_BYTES = 1024 * 1024;
const MAX_CHANGED_PATH_CHARS = 512;
const BASE_REVISION_PATTERN = /^[0-9a-f]{40}$/u;
const CONTROL_CHARACTER_PATTERN = /\p{Cc}/u;
const BACKEND_SOURCE_PREFIX = "src-tauri/src/";
const RUST_FORMAT_CHECK_FAILED = "Rust format check failed";

export function parseChangedRustFiles(raw) {
  if (typeof raw !== "string" || raw.length > MAX_GIT_OUTPUT_BYTES) {
    throw new Error("Git output is too large");
  }
  const files = [];
  let changedCount = 0;
  let start = 0;
  for (let index = 0; index <= raw.length; index += 1) {
    if (index !== raw.length && raw[index] !== "\0") continue;
    const file = raw.slice(start, index);
    start = index + 1;
    if (!file) continue;
    changedCount += 1;
    if (changedCount > MAX_CHANGED_FILES) {
      throw new Error("too many changed files");
    }
    if (
      file.length > MAX_CHANGED_PATH_CHARS
      || file.includes("\\")
      || CONTROL_CHARACTER_PATTERN.test(file)
    ) {
      throw new Error("invalid changed file");
    }
    const segments = file.split("/");
    if (segments.includes("..")) {
      throw new Error("invalid changed file");
    }
    if (
      file.startsWith(BACKEND_SOURCE_PREFIX)
      && file.endsWith(".rs")
    ) {
      files.push(file);
    }
  }
  return files;
}

export function validateBaseRevision(value) {
  if (!BASE_REVISION_PATTERN.test(value)) {
    throw new Error("invalid base revision");
  }
  return value;
}

export function rustfmtArguments(file) {
  return [
    "--edition",
    "2021",
    "--check",
    "--config",
    "skip_children=true",
    file,
  ];
}

function gitChangedFiles(baseRevision) {
  const rangeArguments = baseRevision ? [baseRevision, "HEAD"] : ["HEAD"];
  const result = spawnSync(
    "git",
    [
      "diff",
      "--name-only",
      "--diff-filter=ACMR",
      "-z",
      ...rangeArguments,
      "--",
      "src-tauri/src",
    ],
    {
      cwd: resolve(import.meta.dirname, "../.."),
      encoding: "utf8",
      maxBuffer: MAX_GIT_OUTPUT_BYTES,
      shell: false,
    },
  );
  if (result.status !== 0 || result.error) {
    throw new Error("unable to inspect changed Rust files");
  }
  return parseChangedRustFiles(result.stdout);
}

async function checkFiles(files) {
  const root = resolve(import.meta.dirname, "../..");
  for (const file of files) {
    try {
      await runCommand({
        command: "rustfmt",
        args: rustfmtArguments(file),
        cwd: root,
        timeoutMs: 30_000,
      });
    } catch {
      throw new Error("a changed Rust source file is not formatted");
    }
  }
}

async function main() {
  const rawBase = process.env.RUSTFMT_BASE_SHA?.trim();
  const baseRevision = rawBase && !/^0{40}$/u.test(rawBase)
    ? validateBaseRevision(rawBase)
    : undefined;
  await checkFiles(gitChangedFiles(baseRevision));
}

const entry = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (entry === import.meta.url) {
  main().catch(() => {
    process.stderr.write(`${RUST_FORMAT_CHECK_FAILED}\n`);
    process.exitCode = 1;
  });
}
