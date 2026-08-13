import { spawn } from "node:child_process";
import { realpath, rm } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { resolveCargoTargetDir } from "../cef/tauri-launch.mjs";

export const E2E_BUILD_TIMEOUT_MS = 35 * 60 * 1000;
export const E2E_JOURNEY_TIMEOUT_MS = 10 * 60 * 1000;
const MAX_PROCESS_TIMEOUT_MS = 60 * 60 * 1000;
const PROCESS_TIMEOUT_MESSAGE = "E2E process timeout";
const PROFILE_CLEANUP_MESSAGE = "E2E profile cleanup failed";
const MAX_PROFILE_PATH_CHARS = 32_768;

export async function canonicalE2eRepoRoot(moduleUrl) {
  const candidate = resolve(fileURLToPath(new URL("../..", moduleUrl)));
  return realpath(candidate);
}

export function buildArguments(platform) {
  const bundleArguments = platform === "darwin"
    ? ["--bundles", "app"]
    : ["--no-bundle"];
  return [
    "build", "--debug", "--features", "e2e",
    "--config", "src-tauri/tauri.e2e.conf.json",
    ...bundleArguments,
  ];
}

export function e2eCargoTargetDir(platform, repoRoot, configuredTargetDir) {
  const configured = resolveCargoTargetDir({
    configuredTargetDir,
    platform,
    repoRoot,
  });
  if (configuredTargetDir !== undefined) return configured;
  if (configured !== undefined) return resolve(configured, "e2e");
  return resolve(repoRoot, "src-tauri", "target", "e2e");
}

export function debugBinaryPath(platform, cargoTargetDir) {
  const debugRoot = resolve(cargoTargetDir, "debug");
  if (platform === "darwin") {
    return join(
      debugRoot, "bundle", "macos", "Beaver.app", "Contents", "MacOS", "cl-go-dash",
    );
  }
  const executable = platform === "win32" ? "cl-go-dash.exe" : "cl-go-dash";
  return join(debugRoot, executable);
}

export function isAllowedProfilePath(profilePath, tempPath) {
  return validPath(profilePath)
    && validPath(tempPath)
    && dirname(profilePath) === tempPath
    && /^beaver-e2e-[A-Za-z0-9_-]+$/.test(basename(profilePath));
}

export async function cleanupProfile(profilePath, {
  tempPath,
  hadPriorFailure = false,
  remove = rm,
  report = (message) => process.stderr.write(message),
} = {}) {
  if (
    !isAllowedProfilePath(profilePath, tempPath)
    || typeof hadPriorFailure !== "boolean"
    || typeof remove !== "function"
    || typeof report !== "function"
  ) {
    throw new Error(PROFILE_CLEANUP_MESSAGE);
  }
  try {
    await remove(profilePath, { recursive: true, force: true, maxRetries: 2 });
  } catch {
    if (!hadPriorFailure) throw new Error(PROFILE_CLEANUP_MESSAGE);
    report(`${PROFILE_CLEANUP_MESSAGE} after an earlier failure.\n`);
  }
}

function validPath(value) {
  return typeof value === "string"
    && value.length > 0
    && value.length <= MAX_PROFILE_PATH_CHARS
    && !/[\0\r\n]/u.test(value);
}

export function runCommand(command, args, { cwd, env, timeoutMs }) {
  if (
    !Number.isSafeInteger(timeoutMs)
    || timeoutMs < 1
    || timeoutMs > MAX_PROCESS_TIMEOUT_MS
  ) {
    return Promise.reject(new Error(PROCESS_TIMEOUT_MESSAGE));
  }
  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(command, args, {
      cwd,
      env,
      shell: false,
      stdio: "inherit",
      windowsHide: true,
    });
    const timeout = setTimeout(() => {
      process.stderr.write(`${PROCESS_TIMEOUT_MESSAGE} after ${timeoutMs} ms.\n`);
      child.kill("SIGTERM");
    }, timeoutMs);
    child.once("error", (error) => {
      clearTimeout(timeout);
      rejectRun(error);
    });
    child.once("exit", (code, signal) => {
      clearTimeout(timeout);
      resolveRun(signal === null && code === 0 ? 0 : 1);
    });
  });
}
