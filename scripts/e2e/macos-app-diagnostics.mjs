import {
  closeSync,
  constants,
  fstatSync,
  lstatSync,
  openSync,
  realpathSync,
  writeSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { posix } from "node:path";

const MAX_PATH_CHARS = 4_096;
const MAX_DIAGNOSTIC_FILE_BYTES = 4_096;
const MAX_CAPTURED_LINE_BYTES = 256;
const DIAGNOSTIC_FILE_NAME = "native-app-exit.log";
const E2E_PROFILE_NAME = /^beaver-e2e-[A-Za-z0-9_-]{1,96}$/u;
const SAFE_LIFECYCLE_STAGES = new Set([
  "main-entered",
  "native-prepared",
  "setup-entered",
  "setup-completed",
  "event-loop-entered",
  "event-loop-returned",
]);
const SAFE_RUN_EVENTS = new Set([
  "ready",
  "exit-requested-user",
  "exit-requested-programmatic",
  "window-close-main",
  "exit",
]);
const SAFE_EXIT_SOURCES = new Set([
  "browser-initialization",
  "browser-launch-callback",
  "browser-child-admission",
  "browser-supervision",
]);
const SAFE_EXIT_SIGNALS = new Set([
  "SIGABRT",
  "SIGBUS",
  "SIGILL",
  "SIGKILL",
  "SIGSEGV",
  "SIGTERM",
  "SIGTRAP",
]);
const SAFE_EXIT_SIGNAL_NAMES = new Set([
  ...[...SAFE_EXIT_SIGNALS].map((signal) => signal.toLowerCase()),
  "unknown",
]);
const SAFE_SUPERVISION_FAILURE = /^(?:admission|reaper|external)-(?:[a-z]+)(?:-[a-z]+){0,3}$/u;

export function isSafeAbsolutePosixPath(value) {
  return typeof value === "string"
    && value.length > 0
    && value.length <= MAX_PATH_CHARS
    && !/[\0-\x1f\x7f]/u.test(value)
    && posix.isAbsolute(value)
    && posix.normalize(value) === value
    && !value.split(posix.sep).includes("..");
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

export function safeObservedDiagnostic(line) {
  const lifecycle = line.match(/^\[e2e-lifecycle\] ([a-z-]+)$/u);
  if (lifecycle && SAFE_LIFECYCLE_STAGES.has(lifecycle[1])) return line;
  const runEvent = line.match(/^\[e2e-run-event\] ([a-z-]+)$/u);
  if (runEvent && SAFE_RUN_EVENTS.has(runEvent[1])) return line;
  const exitSource = line.match(/^\[e2e-exit-source\] ([a-z-]+)$/u);
  if (exitSource && SAFE_EXIT_SOURCES.has(exitSource[1])) return line;
  const supervision = line.match(/^\[e2e-supervision-failure\] ([a-z-]{1,64})$/u);
  if (supervision && SAFE_SUPERVISION_FAILURE.test(supervision[1])) return line;
  if (/^\[e2e-process\] application-spawn-failed$/u.test(line)) return line;
  const exitCode = line.match(/^\[e2e-process\] application-exit-code-([0-9]{1,3})$/u);
  if (exitCode && Number(exitCode[1]) <= 255) return line;
  const signal = line.match(/^\[e2e-process\] application-exit-signal-([a-z]+)$/u);
  if (signal && SAFE_EXIT_SIGNAL_NAMES.has(signal[1])) return line;
  return undefined;
}

export function createDiagnosticBuffer(onDiagnostic) {
  let bytes = [];
  let overflowed = false;
  const flush = () => {
    if (!overflowed && bytes.length > 0) {
      const diagnostic = safeObservedDiagnostic(Buffer.from(bytes).toString("utf8"));
      if (diagnostic) onDiagnostic(diagnostic);
    }
    bytes = [];
    overflowed = false;
  };
  return {
    push(chunk) {
      for (const byte of chunk) {
        if (byte === 10) {
          flush();
        } else if (byte !== 13) {
          if (bytes.length < MAX_CAPTURED_LINE_BYTES) bytes.push(byte);
          else overflowed = true;
        }
      }
    },
    finish: flush,
  };
}

export function observeDiagnosticStream(stream) {
  const capture = createDiagnosticBuffer(persistDiagnostic);
  stream.on("data", (chunk) => {
    try {
      writeSync(2, chunk);
    } catch {
      // Persisted fixed categories remain available when stderr forwarding closes early.
    }
    capture.push(chunk);
  });
  stream.once("end", capture.finish);
  stream.once("error", capture.finish);
}

export function writeProcessDiagnostic(message) {
  writeSync(2, `${message}\n`);
  persistDiagnostic(message);
}

function persistDiagnostic(message) {
  if (!safeObservedDiagnostic(message)) return;
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
    // The forwarded stderr marker remains available if persistence is unavailable.
  } finally {
    try {
      if (descriptor !== undefined) closeSync(descriptor);
    } catch {
      // The observer is already exiting and must not expose local filesystem failures.
    }
  }
}
