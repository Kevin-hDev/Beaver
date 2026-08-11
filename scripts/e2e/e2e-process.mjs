import { spawn } from "node:child_process";
import { rm } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { resolveCargoTargetDir } from "../cef/tauri-launch.mjs";

const PROCESS_TIMEOUT_MS = 20 * 60 * 1000;
const PROFILE_CLEANUP_MESSAGE = "E2E profile cleanup failed";
const MAX_PROFILE_PATH_CHARS = 32_768;

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

export function runCommand(command, args, { cwd, env }) {
  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(command, args, {
      cwd,
      env,
      shell: false,
      stdio: "inherit",
      windowsHide: true,
    });
    const timeout = setTimeout(() => child.kill("SIGTERM"), PROCESS_TIMEOUT_MS);
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
