import { createReadStream } from "node:fs";
import { lstat, opendir } from "node:fs/promises";
import { join } from "node:path";
import { createInterface } from "node:readline";

export const MAX_DIAGNOSTIC_FILES = 4;
const MAX_DIRECTORY_ENTRIES = 64;
const MAX_DIAGNOSTIC_FILE_BYTES = 512 * 1024;
const MAX_DIAGNOSTIC_LINES = 2_048;
const MAX_DIAGNOSTICS = 8;
const LOG_NAME = /^wdio[-A-Za-z0-9.]{1,96}\.log$/u;
const SAFE_CATEGORIES = new Set([
  "cef-supervision-object",
  "cef-supervision-permission",
  "cef-supervision-admission",
  "cef-supervision-reaper",
  "cef-supervision-sandbox",
]);

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
    if (entry.isFile() && LOG_NAME.test(entry.name)) names.push(entry.name);
    if (names.length >= MAX_DIAGNOSTIC_FILES) break;
  }
  return names;
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
