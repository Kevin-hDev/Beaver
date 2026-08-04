import { runCommand } from "./command-runner.mjs";

const PYTHON_CHECK = "import sys; raise SystemExit(0 if sys.version_info >= (3, 10) else 1)";
const PROBE_TIMEOUT_MS = 5000;
const MAX_CANDIDATES = 4;
const PYTHON_UNAVAILABLE = "Python runtime unavailable";

const WINDOWS_CANDIDATES = Object.freeze([
  Object.freeze({ command: "py", prefixArgs: Object.freeze(["-3"]) }),
  Object.freeze({ command: "python", prefixArgs: Object.freeze([]) }),
  Object.freeze({ command: "python3", prefixArgs: Object.freeze([]) }),
  Object.freeze({ command: "uv", prefixArgs: Object.freeze(["run", "python"]) }),
]);

const UNIX_CANDIDATES = Object.freeze([
  Object.freeze({ command: "python3", prefixArgs: Object.freeze([]) }),
  Object.freeze({ command: "python", prefixArgs: Object.freeze([]) }),
  Object.freeze({ command: "uv", prefixArgs: Object.freeze(["run", "python"]) }),
]);

function isCandidate(candidate) {
  return (
    typeof candidate === "object" &&
    candidate !== null &&
    typeof candidate.command === "string" &&
    candidate.command.length > 0 &&
    candidate.command.length <= 512 &&
    !/[\0\r\n]/.test(candidate.command) &&
    Array.isArray(candidate.prefixArgs) &&
    candidate.prefixArgs.length <= 4 &&
    candidate.prefixArgs.every(
      (argument) =>
        typeof argument === "string" &&
        argument.length <= 512 &&
        !/[\0\r\n]/.test(argument),
    )
  );
}

export function pythonCandidates(platform) {
  return platform === "win32" ? WINDOWS_CANDIDATES : UNIX_CANDIDATES;
}

export async function probePythonCandidate(candidate, run = runCommand) {
  if (!isCandidate(candidate) || typeof run !== "function") return false;
  try {
    await run({
      command: candidate.command,
      args: [...candidate.prefixArgs, "-c", PYTHON_CHECK],
      cwd: process.cwd(),
      stdio: "ignore",
      timeoutMs: PROBE_TIMEOUT_MS,
    });
    return true;
  } catch {
    return false;
  }
}

export async function resolvePythonCommand(request) {
  let platform;
  let probe;
  try {
    if (typeof request !== "object" || request === null || Array.isArray(request)) {
      throw new Error(PYTHON_UNAVAILABLE);
    }
    platform = request.platform;
    const suppliedProbe = request.probe;
    probe = suppliedProbe === undefined ? probePythonCandidate : suppliedProbe;
  } catch {
    throw new Error(PYTHON_UNAVAILABLE);
  }
  if (typeof platform !== "string" || platform.length > 32 || /[\0\r\n]/.test(platform)) {
    throw new Error(PYTHON_UNAVAILABLE);
  }
  if (typeof probe !== "function") throw new Error(PYTHON_UNAVAILABLE);

  for (const candidate of pythonCandidates(platform).slice(0, MAX_CANDIDATES)) {
    try {
      if (await probe(candidate)) return candidate;
    } catch {
      // A failed probe cannot select an unchecked interpreter.
    }
  }
  throw new Error(PYTHON_UNAVAILABLE);
}
