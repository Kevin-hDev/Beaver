import { spawn } from "node:child_process";
import { existsSync, statSync, writeSync } from "node:fs";
import { posix } from "node:path";
import { pathToFileURL } from "node:url";

const MAX_BINARY_PATH_CHARS = 4_096;
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
  return typeof value === "string"
    && value.length > 0
    && value.length <= MAX_BINARY_PATH_CHARS
    && !/[\0-\x1f\x7f]/u.test(value)
    && posix.isAbsolute(value)
    && posix.normalize(value) === value
    && !value.split(posix.sep).includes("..");
}

export function observedLaunch(command) {
  return {
    command,
    args: [],
    options: {
      env: process.env,
      shell: false,
      stdio: "inherit",
    },
  };
}

export function exitDiagnostic(code, signal) {
  if (Number.isInteger(code) && code >= 0 && code <= 255) {
    return `[e2e-process] application-exit-code-${code}`;
  }
  const safeSignal = SAFE_EXIT_SIGNALS.has(signal) ? signal.toLowerCase() : "unknown";
  return `[e2e-process] application-exit-signal-${safeSignal}`;
}

function run() {
  const binary = process.env.E2E_APP_BINARY;
  if (!isAllowedObservedBinary(binary) || !isExecutableFile(binary)) {
    writeDiagnostic("[e2e-process] application-spawn-failed");
    process.exitCode = 1;
    return;
  }
  const launch = observedLaunch(binary);
  const child = spawn(launch.command, launch.args, launch.options);
  const forwarders = new Map();
  for (const signal of FORWARDED_SIGNALS) {
    const forward = () => child.kill(signal);
    forwarders.set(signal, forward);
    process.once(signal, forward);
  }
  child.once("error", () => {
    removeForwarders(forwarders);
    writeDiagnostic("[e2e-process] application-spawn-failed");
    process.exitCode = 1;
  });
  child.once("exit", (code, signal) => {
    removeForwarders(forwarders);
    writeDiagnostic(exitDiagnostic(code, signal));
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

function writeDiagnostic(message) {
  writeSync(2, `${message}\n`);
}

const invokedPath = process.argv[1];
if (invokedPath && pathToFileURL(invokedPath).href === import.meta.url) run();
