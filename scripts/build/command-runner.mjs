import { lstatSync, realpathSync } from "node:fs";
import { isAbsolute, join, parse, relative, sep } from "node:path";
import { spawn } from "node:child_process";

const MAX_ARGUMENTS = 64;
const MAX_ARGUMENT_LENGTH = 512;
const MAX_COMMAND_LENGTH = 512;
const MAX_CWD_LENGTH = 4096;
const MAX_ENV_ENTRIES = 128;
const MAX_ENV_KEY_LENGTH = 128;
const MAX_ENV_VALUE_LENGTH = 8192;
const MAX_PATH_LENGTH = 8192;
const MAX_TIMEOUT_MS = 300_000;
const BUILD_COMMAND_FAILED = "Build command failed";

function invalidSpec() {
  throw new Error(BUILD_COMMAND_FAILED);
}

function hasControlCharacters(value) {
  return value.includes("\0") || value.includes("\r") || value.includes("\n");
}

function isSafeString(value, maximumLength, allowEmpty = false) {
  return (
    typeof value === "string" &&
    (allowEmpty || value.length > 0) &&
    value.length <= maximumLength &&
    !hasControlCharacters(value)
  );
}

function hasTraversalSegment(value) {
  return value.split(/[\\/]+/).includes("..");
}

function comparablePath(value) {
  if (process.platform !== "win32") return value;
  return value
    .replace(/^\\\\\?\\UNC\\/i, "\\\\")
    .replace(/^\\\\\?\\/i, "")
    .toLowerCase();
}

function containsLink(cwd) {
  const root = parse(cwd).root;
  const segments = relative(root, cwd).split(sep).filter(Boolean);
  let current = root;
  for (const segment of segments) {
    current = join(current, segment);
    if (lstatSync(current).isSymbolicLink()) return true;
  }
  return false;
}

function validateCwd(cwd) {
  if (!isSafeString(cwd, MAX_CWD_LENGTH) || !isAbsolute(cwd) || hasTraversalSegment(cwd)) {
    invalidSpec();
  }
  const canonical = realpathSync.native(cwd);
  if (comparablePath(cwd) !== comparablePath(canonical) || containsLink(cwd)) invalidSpec();
}

function validateEnvironment(env) {
  if (typeof env !== "object" || env === null || Array.isArray(env)) invalidSpec();
  let entries = 0;
  let pathCount = 0;
  for (const key in env) {
    entries += 1;
    if (entries > MAX_ENV_ENTRIES) invalidSpec();
    const value = env[key];
    if (!isSafeString(key, MAX_ENV_KEY_LENGTH) || !isSafeString(value, MAX_ENV_VALUE_LENGTH, true)) {
      invalidSpec();
    }
    if (key.toUpperCase() === "PATH") {
      pathCount += 1;
      if (value.length > MAX_PATH_LENGTH) invalidSpec();
    }
  }
  if (pathCount > 1) invalidSpec();
}

function copyEnvironment(env) {
  if (typeof env !== "object" || env === null || Array.isArray(env)) invalidSpec();
  const copied = Object.create(null);
  let entries = 0;
  for (const key in env) {
    entries += 1;
    if (entries > MAX_ENV_ENTRIES) invalidSpec();
    copied[key] = env[key];
  }
  return copied;
}

function snapshotCommandSpec(spec) {
  if (typeof spec !== "object" || spec === null || Array.isArray(spec)) invalidSpec();
  const sourceArgs = spec.args;
  if (!Array.isArray(sourceArgs) || !Number.isSafeInteger(sourceArgs.length) || sourceArgs.length > MAX_ARGUMENTS) {
    invalidSpec();
  }
  const args = new Array(sourceArgs.length);
  for (let index = 0; index < args.length; index += 1) args[index] = sourceArgs[index];

  const sourceEnv = spec.env;
  return {
    command: spec.command,
    args,
    cwd: spec.cwd,
    env: sourceEnv === undefined ? undefined : copyEnvironment(sourceEnv),
    stdio: spec.stdio,
    timeoutMs: spec.timeoutMs,
  };
}

export function validateCommandSpec(spec) {
  try {
    if (typeof spec !== "object" || spec === null || Array.isArray(spec)) invalidSpec();
    if (!isSafeString(spec.command, MAX_COMMAND_LENGTH)) invalidSpec();
    if (!Array.isArray(spec.args) || spec.args.length > MAX_ARGUMENTS) invalidSpec();
    validateCwd(spec.cwd);
    if (!spec.args.every((argument) => isSafeString(argument, MAX_ARGUMENT_LENGTH, true))) invalidSpec();
    if (spec.stdio !== undefined && spec.stdio !== "inherit" && spec.stdio !== "ignore") invalidSpec();
    if (spec.timeoutMs !== undefined && (!Number.isSafeInteger(spec.timeoutMs) || spec.timeoutMs < 1 || spec.timeoutMs > MAX_TIMEOUT_MS)) {
      invalidSpec();
    }
    validateEnvironment(spec.env === undefined ? process.env : spec.env);
  } catch {
    invalidSpec();
  }
}

export function runCommand(spec, spawnProcess = spawn) {
  let safeSpec;
  try {
    safeSpec = snapshotCommandSpec(spec);
    validateCommandSpec(safeSpec);
  } catch {
    return Promise.reject(new Error(BUILD_COMMAND_FAILED));
  }
  return new Promise((resolve, reject) => {
    let settled = false;
    let timeout;
    let child;
    const onError = () => settle(true);
    const onExit = (code, signal) => settle(signal !== null || code !== 0);
    const removeListener = (name, handler) => {
      if (typeof child?.off === "function") child.off(name, handler);
      else if (typeof child?.removeListener === "function") child.removeListener(name, handler);
    };
    const cleanup = () => {
      if (timeout !== undefined) clearTimeout(timeout);
      timeout = undefined;
      removeListener("error", onError);
      removeListener("exit", onExit);
    };
    const settle = (error) => {
      if (settled) return;
      settled = true;
      cleanup();
      if (error) reject(new Error(BUILD_COMMAND_FAILED));
      else resolve();
    };

    try {
      child = spawnProcess(safeSpec.command, safeSpec.args, {
        cwd: safeSpec.cwd,
        env: safeSpec.env === undefined ? process.env : safeSpec.env,
        shell: false,
        stdio: safeSpec.stdio ?? "inherit",
        windowsHide: true,
      });
      child.once("error", onError);
      child.once("exit", onExit);
      if (!settled && safeSpec.timeoutMs !== undefined) {
        timeout = setTimeout(() => {
          try {
            child.kill();
          } catch {
            // The timeout still rejects even when the child cannot be terminated.
          }
          settle(true);
        }, safeSpec.timeoutMs);
      }
    } catch {
      settle(true);
    }
  });
}
