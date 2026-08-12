import { spawnSync } from "node:child_process";
import { posix, win32 } from "node:path";

const MAX_PROCESS_OUTPUT_BYTES = 4 * 1024 * 1024;
const MAX_PROCESSES = 4_096;
const MAX_COMMAND_CHARS = 65_536;
const MAX_WAIT_MS = 30_000;
const MAX_POLL_MS = 1_000;
const MAX_REQUESTED_PIDS = 64;
const FAILURE_MESSAGE = "Native CEF observation failed";

function invalid(condition) {
  if (condition) throw new Error(FAILURE_MESSAGE);
}

function processRecord(pid, parentPid, executable, command) {
  if (!Number.isSafeInteger(pid) || pid <= 0 || pid > 0xffff_ffff) return null;
  if (!Number.isSafeInteger(parentPid) || parentPid < 0 || parentPid > 0xffff_ffff) return null;
  if (typeof executable !== "string" || executable.length > MAX_COMMAND_CHARS) return null;
  if (typeof command !== "string" || command.length > MAX_COMMAND_CHARS) return null;
  return { pid, parentPid, executable, command };
}

export function parseWindowsProcessJson(output) {
  invalid(typeof output !== "string" || Buffer.byteLength(output) > MAX_PROCESS_OUTPUT_BYTES);
  if (output.trim() === "") return [];
  let decoded;
  try {
    decoded = JSON.parse(output);
  } catch {
    throw new Error(FAILURE_MESSAGE);
  }
  const entries = Array.isArray(decoded) ? decoded : [decoded];
  invalid(entries.length > MAX_PROCESSES);
  return entries.flatMap((entry) => {
    if (!entry || typeof entry !== "object") return [];
    const record = processRecord(
      entry.ProcessId,
      entry.ParentProcessId ?? 0,
      entry.ExecutablePath ?? "",
      entry.CommandLine ?? "",
    );
    return record ? [record] : [];
  });
}

export function parseMacProcessTable(output) {
  invalid(typeof output !== "string" || Buffer.byteLength(output) > MAX_PROCESS_OUTPUT_BYTES);
  const lines = output.split(/\r?\n/u).filter(Boolean);
  invalid(lines.length > MAX_PROCESSES);
  return lines.flatMap((line) => {
    const match = /^\s*(\d+)\s+(\d+)\s+([\s\S]*)$/u.exec(line);
    if (!match) return [];
    const record = processRecord(Number(match[1]), Number(match[2]), "", match[3]);
    return record ? [record] : [];
  });
}

export function parseLinuxProcessTable(output) {
  invalid(typeof output !== "string" || Buffer.byteLength(output) > MAX_PROCESS_OUTPUT_BYTES);
  const lines = output.split(/\r?\n/u).filter(Boolean);
  invalid(lines.length > MAX_PROCESSES);
  return lines.flatMap((line) => {
    const match = /^\s*(\d+)\s+(\d+)\s+(\S+)\s+([\s\S]*)$/u.exec(line);
    if (!match) return [];
    const record = processRecord(Number(match[1]), Number(match[2]), match[3], match[4]);
    return record ? [record] : [];
  });
}

export function listNativeProcesses(platform = process.platform) {
  if (platform === "win32") {
    const script = "Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine | ConvertTo-Json -Compress";
    return parseWindowsProcessJson(run("powershell.exe", [
      "-NoProfile", "-NonInteractive", "-Command", script,
    ]));
  }
  if (platform === "darwin") {
    return parseMacProcessTable(run("/bin/ps", ["-axo", "pid=,ppid=,command="]));
  }
  if (platform === "linux") {
    return parseLinuxProcessTable(run("/bin/ps", ["-axo", "pid=,ppid=,comm=,args="]));
  }
  throw new Error(FAILURE_MESSAGE);
}

function run(command, args) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    maxBuffer: MAX_PROCESS_OUTPUT_BYTES,
    shell: false,
    windowsHide: true,
  });
  invalid(result.error || result.signal || result.status !== 0);
  return result.stdout;
}

function isContainedExecutable(process, root, platform) {
  if (platform === "win32") {
    if (!process.executable) return false;
    const normalizedRoot = win32.resolve(root).toLocaleLowerCase("en-US");
    const executable = win32.resolve(process.executable).toLocaleLowerCase("en-US");
    const relative = win32.relative(normalizedRoot, executable);
    return relative === "" || (!relative.startsWith("..") && !win32.isAbsolute(relative));
  }
  if (platform === "darwin") {
    const normalizedRoot = posix.resolve(root);
    return process.command === normalizedRoot
      || process.command.startsWith(`${normalizedRoot}/`);
  }
  return false;
}

export function hasOwnedProcess(processes, root, platform = process.platform) {
  return processes.some((entry) => isContainedExecutable(entry, root, platform));
}

export function hasOwnedCefHelper(processes, root, platform = process.platform) {
  return processes.some((entry) => isContainedExecutable(entry, root, platform)
    && /(?:^|\s)--type=[^\s]+/u.test(entry.command));
}

export function assertOwnedCefHelpersSandboxed(
  processes,
  root,
  platform = process.platform,
) {
  const insecure = processes.some((entry) => isContainedExecutable(entry, root, platform)
    && /(?:^|\s)--type=[^\s]+/u.test(entry.command)
    && /(?:^|\s)--no-sandbox(?:=|\s|$)/u.test(entry.command));
  invalid(insecure);
}

export function runtimeRootForBinary(platform, binaryPath) {
  invalid(typeof binaryPath !== "string" || binaryPath.length === 0 || binaryPath.length > 1_024);
  if (platform === "win32") {
    invalid(!win32.isAbsolute(binaryPath));
    return win32.dirname(binaryPath);
  }
  if (platform === "darwin") {
    const marker = ".app/Contents/MacOS/";
    const markerIndex = binaryPath.lastIndexOf(marker);
    invalid(!posix.isAbsolute(binaryPath) || markerIndex < 1);
    return binaryPath.slice(0, markerIndex + ".app".length);
  }
  throw new Error(FAILURE_MESSAGE);
}

function validateWait({ root, timeoutMs, pollMs, listProcesses }) {
  invalid(typeof root !== "string" || root.length === 0 || root.length > 1_024);
  invalid(!Number.isInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > MAX_WAIT_MS);
  invalid(!Number.isInteger(pollMs) || pollMs < 1 || pollMs > MAX_POLL_MS);
  invalid(typeof listProcesses !== "function");
}

async function waitFor(predicate, options) {
  validateWait(options);
  const deadline = Date.now() + options.timeoutMs;
  do {
    const processes = await options.listProcesses(options.platform);
    invalid(!Array.isArray(processes) || processes.length > MAX_PROCESSES);
    if (predicate(processes, options.root, options.platform)) return;
    if (Date.now() >= deadline) break;
    await new Promise((resolve) => setTimeout(resolve, options.pollMs));
  } while (Date.now() <= deadline);
  throw new Error(FAILURE_MESSAGE);
}

export function waitForOwnedCefHelper({
  platform = process.platform,
  root,
  timeoutMs = 15_000,
  pollMs = 100,
  listProcesses = listNativeProcesses,
}) {
  return waitFor((processes, ownedRoot, ownedPlatform) => {
    assertOwnedCefHelpersSandboxed(processes, ownedRoot, ownedPlatform);
    return hasOwnedCefHelper(processes, ownedRoot, ownedPlatform);
  }, { platform, root, timeoutMs, pollMs, listProcesses });
}

export function waitForOwnedProcessesToExit({
  platform = process.platform,
  root,
  timeoutMs = 20_000,
  pollMs = 100,
  listProcesses = listNativeProcesses,
}) {
  return waitFor((processes, ownedRoot, ownedPlatform) => (
    !hasOwnedProcess(processes, ownedRoot, ownedPlatform)
  ), { platform, root, timeoutMs, pollMs, listProcesses });
}

export async function waitForProcessIdsToExit({
  platform = process.platform,
  pids,
  timeoutMs = 20_000,
  pollMs = 100,
  listProcesses = listNativeProcesses,
}) {
  invalid(!Array.isArray(pids) || pids.length > MAX_REQUESTED_PIDS);
  invalid(pids.some((pid) => !Number.isSafeInteger(pid) || pid < 2 || pid > 0xffff_ffff));
  invalid(new Set(pids).size !== pids.length);
  invalid(!Number.isInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > MAX_WAIT_MS);
  invalid(!Number.isInteger(pollMs) || pollMs < 1 || pollMs > MAX_POLL_MS);
  invalid(typeof listProcesses !== "function");
  const expected = new Set(pids);
  const deadline = Date.now() + timeoutMs;
  do {
    const processes = await listProcesses(platform);
    invalid(!Array.isArray(processes) || processes.length > MAX_PROCESSES);
    if (!processes.some(({ pid }) => expected.has(pid))) return;
    if (Date.now() >= deadline) break;
    await new Promise((resolve) => setTimeout(resolve, pollMs));
  } while (Date.now() <= deadline);
  throw new Error(FAILURE_MESSAGE);
}
