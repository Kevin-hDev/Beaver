import { spawn } from "node:child_process";
import {
  closeSync,
  constants,
  existsSync,
  fstatSync,
  lstatSync,
  openSync,
  realpathSync,
  statSync,
  writeSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { posix } from "node:path";
import { pathToFileURL } from "node:url";

const MAX_BINARY_PATH_CHARS = 4_096;
const MAX_DIAGNOSTIC_FILE_BYTES = 4_096;
const DIAGNOSTIC_FILE_NAME = "native-app-exit.log";
const E2E_PROFILE_NAME = /^beaver-e2e-[A-Za-z0-9_-]{1,96}$/u;
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

export function diagnosticFilePath(
  logDirectory,
  { realpath = realpathSync, temporaryDirectory = tmpdir() } = {},
) {
  if (!isSafeAbsolutePosixPath(logDirectory)
    || posix.basename(logDirectory) !== "logs") {
    return undefined;
  }
  try {
    const profileDirectory = posix.dirname(logDirectory);
    const canonicalTemporaryDirectory = realpath(temporaryDirectory);
    const canonicalProfileDirectory = realpath(profileDirectory);
    const canonicalLogDirectory = realpath(logDirectory);
    if (!isSafeAbsolutePosixPath(canonicalTemporaryDirectory)
      || !isSafeAbsolutePosixPath(canonicalProfileDirectory)
      || !isSafeAbsolutePosixPath(canonicalLogDirectory)
      || canonicalProfileDirectory !== profileDirectory
      || canonicalLogDirectory !== logDirectory
      || posix.dirname(canonicalProfileDirectory) !== canonicalTemporaryDirectory
      || !E2E_PROFILE_NAME.test(posix.basename(canonicalProfileDirectory))
      || canonicalLogDirectory !== posix.join(canonicalProfileDirectory, "logs")) {
      return undefined;
    }
    return posix.join(canonicalLogDirectory, DIAGNOSTIC_FILE_NAME);
  } catch {
    return undefined;
  }
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
  persistDiagnostic(message);
}

function persistDiagnostic(message) {
  if (!isSafeDiagnostic(message)) return;
  const path = diagnosticFilePath(process.env.E2E_LOG_DIR);
  if (!path) return;
  let descriptor;
  try {
    const directoryMetadata = lstatSync(posix.dirname(path));
    if (!directoryMetadata.isDirectory() || directoryMetadata.isSymbolicLink()) return;
    const payload = `${message}\n`;
    descriptor = openSync(
      path,
      constants.O_CREAT
        | constants.O_WRONLY
        | constants.O_APPEND
        | (constants.O_NOFOLLOW ?? 0),
      0o600,
    );
    const metadata = fstatSync(descriptor);
    if (!metadata.isFile()
      || metadata.size + Buffer.byteLength(payload, "utf8") > MAX_DIAGNOSTIC_FILE_BYTES) {
      return;
    }
    writeSync(descriptor, payload);
  } catch {
    // The stderr marker remains available when this bounded diagnostic cannot be persisted.
  } finally {
    try {
      if (descriptor !== undefined) closeSync(descriptor);
    } catch {
      // The observer is already exiting and must never expose a local filesystem failure.
    }
  }
}

function isSafeAbsolutePosixPath(value) {
  return typeof value === "string"
    && value.length > 0
    && value.length <= MAX_BINARY_PATH_CHARS
    && !/[\0-\x1f\x7f]/u.test(value)
    && posix.isAbsolute(value)
    && posix.normalize(value) === value
    && !value.split(posix.sep).includes("..");
}

function isSafeDiagnostic(message) {
  return /^\[e2e-process\] application-(?:spawn-failed|exit-code-[0-9]{1,3}|exit-signal-[a-z]+)$/u
    .test(message);
}

const invokedPath = process.argv[1];
if (invokedPath && pathToFileURL(invokedPath).href === import.meta.url) run();
