import { lstatSync, readFileSync, realpathSync, statSync } from "node:fs";
import { isAbsolute, join, relative } from "node:path";

import { runCommand } from "./command-runner.mjs";

const PYTHON_UNAVAILABLE = "Python runtime unavailable";
const MAX_VERSION_FILE_BYTES = 32;
const PROBE_TIMEOUT_MS = 5000;
const MAX_CANDIDATES = 4;
// One controlled file keeps preparation and release on the same CPython minor version.
const VERSION_FILE = "scripts/build/searxng-python-version.txt";

function unavailable() {
  throw new Error(PYTHON_UNAVAILABLE);
}

function isExpectedVersion(value) {
  return (
    typeof value === "object" && value !== null &&
    Number.isInteger(value.major) && value.major === 3 &&
    Number.isInteger(value.minor) && value.minor >= 10 && value.minor <= 99 &&
    value.label === `3.${value.minor}`
  );
}

function isCandidate(candidate) {
  return (
    typeof candidate === "object" && candidate !== null &&
    typeof candidate.command === "string" && candidate.command.length > 0 &&
    candidate.command.length <= 512 && !/[\0\r\n]/u.test(candidate.command) &&
    typeof candidate.label === "string" && candidate.label.length > 0 &&
    candidate.label.length <= 512 && !/[\0\r\n]/u.test(candidate.label) &&
    Array.isArray(candidate.prefixArgs) && candidate.prefixArgs.length <= 4 &&
    candidate.prefixArgs.every((argument) => (
      typeof argument === "string" && argument.length <= 512 && !/[\0\r\n]/u.test(argument)
    ))
  );
}

function isInside(root, target) {
  const remainder = relative(root, target);
  return remainder !== "" && !remainder.startsWith("..") && !isAbsolute(remainder);
}

function containsLink(root, target) {
  let current = root;
  for (const segment of relative(root, target).split(/[\\/]+/u).filter(Boolean)) {
    current = join(current, segment);
    if (lstatSync(current).isSymbolicLink()) return true;
  }
  return false;
}

export function parseSupportedPythonVersion(body) {
  if (typeof body !== "string" || Buffer.byteLength(body) > MAX_VERSION_FILE_BYTES) unavailable();
  const match = /^3\.(1[0-9]|[2-9][0-9])\n?$/u.exec(body);
  if (!match) unavailable();
  const minor = Number.parseInt(match[1], 10);
  return Object.freeze({ major: 3, minor, label: `3.${minor}` });
}

export function readSupportedPythonVersion(repoRoot) {
  try {
    if (typeof repoRoot !== "string" || repoRoot.length === 0 || repoRoot.length > 4096) unavailable();
    const root = realpathSync.native(repoRoot);
    const versionFile = join(root, VERSION_FILE);
    const canonicalVersionFile = realpathSync.native(versionFile);
    if (
      !statSync(root).isDirectory() || !statSync(canonicalVersionFile).isFile() ||
      !isInside(root, canonicalVersionFile) || containsLink(root, versionFile) ||
      statSync(canonicalVersionFile).size > MAX_VERSION_FILE_BYTES
    ) unavailable();
    return parseSupportedPythonVersion(readFileSync(canonicalVersionFile, "utf8"));
  } catch {
    unavailable();
  }
}

export function pythonCandidates(platform, expectedVersion) {
  if (!isExpectedVersion(expectedVersion)) unavailable();
  const exact = expectedVersion.label;
  return platform === "win32"
    ? [
      { command: "py", prefixArgs: [`-${exact}`], label: `py-${exact}` },
      { command: `python${exact}`, prefixArgs: [], label: `python${exact}` },
      { command: "python3", prefixArgs: [], label: "python3" },
      { command: "python", prefixArgs: [], label: "python" },
    ]
    : [
      { command: `python${exact}`, prefixArgs: [], label: `python${exact}` },
      { command: "python3", prefixArgs: [], label: "python3" },
      { command: "python", prefixArgs: [], label: "python" },
    ];
}

export async function probePythonCandidate(candidate, expectedVersion, run = runCommand) {
  if (!isCandidate(candidate) || !isExpectedVersion(expectedVersion) || typeof run !== "function") return false;
  const pythonCheck = [
    "import sys",
    `raise SystemExit(0 if (sys.implementation.name, sys.version_info.major, sys.version_info.minor) == ('cpython', ${expectedVersion.major}, ${expectedVersion.minor}) else 1)`,
  ].join("; ");
  try {
    await run({
      command: candidate.command,
      args: [...candidate.prefixArgs, "-c", pythonCheck],
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
  if (typeof request !== "object" || request === null || Array.isArray(request)) unavailable();
  const { platform, expectedVersion, probe = probePythonCandidate } = request;
  if (
    typeof platform !== "string" || platform.length > 32 || /[\0\r\n]/u.test(platform) ||
    !isExpectedVersion(expectedVersion) || typeof probe !== "function"
  ) unavailable();
  for (const candidate of pythonCandidates(platform, expectedVersion).slice(0, MAX_CANDIDATES)) {
    try {
      if (await probe(candidate, expectedVersion)) return Object.freeze(candidate);
    } catch {
      // A failed probe cannot select an unchecked interpreter.
    }
  }
  unavailable();
}
