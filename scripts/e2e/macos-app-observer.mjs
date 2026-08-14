import { spawn } from "node:child_process";
import { existsSync, statSync } from "node:fs";
import { pathToFileURL } from "node:url";
import {
  exitDiagnostic,
  isSafeAbsolutePosixPath,
  observeDiagnosticStream,
  writeProcessDiagnostic,
} from "./macos-app-diagnostics.mjs";
import { captureMacCefTurnoverProof } from "./macos-cef-turnover-proof.mjs";
import { NATIVE_CEF_STAGE_CEILINGS_MS } from "./native-journey-deadline.mjs";
import { runtimeRootForBinary } from "./native-cef-runtime-root.mjs";

const FORWARDED_SIGNALS = ["SIGHUP", "SIGINT", "SIGTERM"];
const SAFE_EXIT_SIGNALS = new Set([
  "SIGABRT",
  "SIGBUS",
  "SIGILL",
  "SIGKILL",
  "SIGSEGV",
  "SIGTERM",
  "SIGTRAP",
]);

export function isAllowedObservedBinary(value) {
  return isSafeAbsolutePosixPath(value);
}

export function observedLaunch(command) {
  return {
    command,
    args: [],
    options: {
      env: process.env,
      shell: false,
      stdio: ["ignore", "inherit", "pipe"],
    },
  };
}

function run() {
  const binary = process.env.E2E_APP_BINARY;
  if (!isAllowedObservedBinary(binary) || !isExecutableFile(binary)) {
    writeProcessDiagnostic("[e2e-process] application-spawn-failed");
    process.exitCode = 1;
    return;
  }
  const launch = observedLaunch(binary);
  if (process.env.E2E_REQUIRE_CEF_SMOKE === "1") {
    void captureMacCefTurnoverProof({
      logDirectory: process.env.E2E_LOG_DIR,
      root: runtimeRootForBinary("darwin", binary),
      timeoutMs: NATIVE_CEF_STAGE_CEILINGS_MS.cef_helper_turnover,
    }).catch(() => {
      writeProcessDiagnostic("[e2e-process] cef-turnover-observation-failed");
    });
  }
  const child = spawn(launch.command, launch.args, launch.options);
  if (child.stderr) observeDiagnosticStream(child.stderr);
  const forwarders = new Map();
  for (const signal of FORWARDED_SIGNALS) {
    const forward = () => child.kill(signal);
    forwarders.set(signal, forward);
    process.once(signal, forward);
  }
  child.once("error", () => {
    removeForwarders(forwarders);
    writeProcessDiagnostic("[e2e-process] application-spawn-failed");
    process.exitCode = 1;
  });
  child.once("exit", (code, signal) => {
    removeForwarders(forwarders);
    writeProcessDiagnostic(exitDiagnostic(code, signal));
    if (SAFE_EXIT_SIGNALS.has(signal)) {
      process.kill(process.pid, signal);
    } else {
      process.exitCode = Number.isInteger(code) ? code : 1;
    }
  });
}

function isExecutableFile(path) {
  try {
    return existsSync(path) && statSync(path).isFile();
  } catch {
    return false;
  }
}

function removeForwarders(forwarders) {
  for (const [signal, forward] of forwarders) process.removeListener(signal, forward);
}

const invokedPath = process.argv[1];
if (invokedPath && pathToFileURL(invokedPath).href === import.meta.url) run();
