import { spawn } from "node:child_process";
import { basename, dirname, join, resolve } from "node:path";

const PROCESS_TIMEOUT_MS = 20 * 60 * 1000;

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

export function debugBinaryPath(platform, repoRoot) {
  const debugRoot = resolve(repoRoot, "src-tauri", "target", "e2e", "debug");
  if (platform === "darwin") {
    return join(
      debugRoot, "bundle", "macos", "Beaver.app", "Contents", "MacOS", "cl-go-dash",
    );
  }
  const executable = platform === "win32" ? "cl-go-dash.exe" : "cl-go-dash";
  return join(debugRoot, executable);
}

export function isAllowedProfilePath(profilePath, tempPath) {
  return dirname(profilePath) === tempPath
    && /^beaver-e2e-[A-Za-z0-9_-]+$/.test(basename(profilePath));
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
