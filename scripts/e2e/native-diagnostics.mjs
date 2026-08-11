import { createReadStream } from "node:fs";
import { lstat, opendir } from "node:fs/promises";
import { join } from "node:path";
import { createInterface } from "node:readline";

export const MAX_DIAGNOSTIC_FILES = 4;
const MAX_DIRECTORY_ENTRIES = 64;
const MAX_DIAGNOSTIC_FILE_BYTES = 512 * 1024;
const MAX_DIAGNOSTIC_LINES = 2_048;
const MAX_DIAGNOSTICS = 16;
const EXIT_LOG_NAME = "native-app-exit.log";
const LOG_NAME = /^wdio[-A-Za-z0-9.]{1,96}\.log$/u;
const SAFE_CATEGORIES = new Set([
  "cef-supervision-object",
  "cef-supervision-permission",
  "cef-supervision-admission",
  "cef-supervision-reaper",
  "cef-supervision-sandbox",
]);
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
  "sigabrt",
  "sigbus",
  "sigill",
  "sigkill",
  "sigsegv",
  "sigterm",
  "sigtrap",
  "unknown",
]);
const SAFE_SUPERVISION_FAILURE = /^(?:admission|reaper|external)-(?:[a-z]+)(?:-[a-z]+){0,3}$/u;

export async function collectNativeCefDiagnostics(logDirectory) {
  const names = await boundedLogNames(logDirectory);
  const diagnostics = new Set();
  for (const name of names) {
    await scanLog(join(logDirectory, name), diagnostics);
    if (diagnostics.size >= MAX_DIAGNOSTICS) break;
  }
  return [...diagnostics];
}

export async function reportNativeCefDiagnostics(
  logDirectory,
  report = process.stderr.write.bind(process.stderr),
) {
  const diagnostics = await collectNativeCefDiagnostics(logDirectory);
  if (diagnostics.length === 0) {
    report("Native CEF diagnostic: no safe browser failure category captured.\n");
    return;
  }
  for (const diagnostic of diagnostics) {
    report(`Native CEF diagnostic: ${diagnostic}\n`);
  }
}

async function boundedLogNames(logDirectory) {
  let directory;
  try {
    directory = await opendir(logDirectory);
  } catch (error) {
    if (error?.code === "ENOENT") return [];
    throw error;
  }
  const names = [];
  let inspected = 0;
  for await (const entry of directory) {
    inspected += 1;
    if (inspected > MAX_DIRECTORY_ENTRIES) break;
    if (entry.isFile() && (entry.name === EXIT_LOG_NAME || LOG_NAME.test(entry.name))) {
      names.push(entry.name);
    }
  }
  names.sort((left, right) => {
    if (left === EXIT_LOG_NAME) return -1;
    if (right === EXIT_LOG_NAME) return 1;
    return left.localeCompare(right);
  });
  return names.slice(0, MAX_DIAGNOSTIC_FILES);
}

async function scanLog(path, diagnostics) {
  const metadata = await lstat(path);
  if (!metadata.isFile() || metadata.size <= 0 || metadata.size > MAX_DIAGNOSTIC_FILE_BYTES) {
    return;
  }
  const lines = createInterface({
    input: createReadStream(path, {
      encoding: "utf8",
      end: MAX_DIAGNOSTIC_FILE_BYTES - 1,
    }),
    crlfDelay: Infinity,
  });
  let inspected = 0;
  for await (const line of lines) {
    inspected += 1;
    if (inspected > MAX_DIAGNOSTIC_LINES) break;
    const diagnostic = safeDiagnostic(line);
    if (diagnostic) diagnostics.add(diagnostic);
    if (diagnostics.size >= MAX_DIAGNOSTICS) break;
  }
  lines.close();
}

function safeDiagnostic(line) {
  if (line.includes("Tauri app spawned (PID:")) {
    return "webdriver:spawned";
  }
  if (line.includes("WebDriver server ready on port")) {
    return "webdriver:ready";
  }
  if (line.includes("Embedded WebDriver on port") && line.includes("is healthy")) {
    return "webdriver:healthy";
  }
  if (line.includes("[exit] coordinated shutdown requested")) {
    return "application-exit:coordinated";
  }
  if (line.includes("[exit] event loop returned")) {
    return "application-exit:event-loop";
  }
  if (line.includes("[browser] launch callback failed")) {
    return "browser-callback:fatal";
  }
  const lifecycle = line.match(/\[e2e-lifecycle\] ([a-z-]+)/u);
  if (lifecycle && SAFE_LIFECYCLE_STAGES.has(lifecycle[1])) {
    return `application-stage:${lifecycle[1]}`;
  }
  const runEvent = line.match(/\[e2e-run-event\] ([a-z-]+)/u);
  if (runEvent && SAFE_RUN_EVENTS.has(runEvent[1])) {
    return `application-event:${runEvent[1]}`;
  }
  const exitSource = line.match(/\[e2e-exit-source\] ([a-z-]+)/u);
  if (exitSource && SAFE_EXIT_SOURCES.has(exitSource[1])) {
    return `application-exit-source:${exitSource[1]}`;
  }
  const supervisionFailure = line.match(
    /\[e2e-supervision-failure\] ([a-z-]{1,64})/u,
  );
  if (supervisionFailure && SAFE_SUPERVISION_FAILURE.test(supervisionFailure[1])) {
    return `browser-supervision-detail:${supervisionFailure[1]}`;
  }
  const exitCode = line.match(/\[e2e-process\] application-exit-code-([0-9]{1,3})/u);
  if (exitCode && Number(exitCode[1]) <= 255) {
    return `process-exit:code-${exitCode[1]}`;
  }
  const exitSignal = line.match(/\[e2e-process\] application-exit-signal-([a-z]+)/u);
  if (exitSignal && SAFE_EXIT_SIGNALS.has(exitSignal[1])) {
    return `process-exit:signal-${exitSignal[1]}`;
  }
  if (line.includes("[e2e-process] application-spawn-failed")) {
    return "process-exit:spawn-failed";
  }
  const helper = line.match(
    /\[browser-helper\] setup failed \((cef-supervision-[a-z-]+)\)/u,
  );
  if (helper && SAFE_CATEGORIES.has(helper[1])) {
    return `browser-helper:${helper[1]}`;
  }
  const supervision = line.match(
    /\[browser\] macOS supervision failed \((cef-supervision-[a-z-]+)\)/u,
  );
  if (supervision && SAFE_CATEGORIES.has(supervision[1])) {
    return `browser-supervision:${supervision[1]}`;
  }
  const preflight = line.match(
    /\[browser\] preflight unavailable \((cef-supervision-[a-z-]+)\)/u,
  );
  if (preflight && SAFE_CATEGORIES.has(preflight[1])) {
    return `browser-preflight:${preflight[1]}`;
  }
  if (line.includes("[browser] initialization failed after CEF boundary")) {
    return "browser-initialization:fatal";
  }
  return undefined;
}
