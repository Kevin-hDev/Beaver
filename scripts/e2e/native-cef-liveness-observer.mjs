import {
  assertOwnedCefHelpersSandboxed,
  listNativeProcesses,
  ownedCefHelperPids,
} from "./native-cef-observer.mjs";

const MAX_TRACKED_HELPERS = 64;
const MAX_WAIT_MS = 30_000;
const MAX_POLL_MS = 1_000;
const FAILURE_MESSAGE = "Native CEF liveness observation failed";

export const CEF_HELPER_TURNOVER_POLL_MS = 25;

export async function waitForOwnedCefHelperSet({
  platform = process.platform,
  root,
  timeoutMs = 15_000,
  pollMs = CEF_HELPER_TURNOVER_POLL_MS,
  listProcesses = listNativeProcesses,
}) {
  validateOptions({ platform, root, timeoutMs, pollMs, listProcesses });
  const deadline = Date.now() + timeoutMs;
  do {
    const processes = await listProcesses(platform);
    assertOwnedCefHelpersSandboxed(processes, root, platform);
    const pids = ownedCefHelperPids(processes, root, platform);
    invalid(pids.length > MAX_TRACKED_HELPERS);
    if (pids.length > 0) return pids;
    if (Date.now() >= deadline) break;
    await delay(pollMs);
  } while (Date.now() <= deadline);
  throw new Error(FAILURE_MESSAGE);
}

export async function waitForOwnedCefHelperTurnover({
  platform = process.platform,
  root,
  initialPids,
  timeoutMs = 20_000,
  pollMs = CEF_HELPER_TURNOVER_POLL_MS,
  listProcesses = listNativeProcesses,
}) {
  validateOptions({ platform, root, timeoutMs, pollMs, listProcesses });
  const initial = validateInitialPids(initialPids);
  const seenPids = new Set(initial);
  const deadline = Date.now() + timeoutMs;
  do {
    const processes = await listProcesses(platform);
    assertOwnedCefHelpersSandboxed(processes, root, platform);
    const currentPids = ownedCefHelperPids(processes, root, platform);
    const current = new Set(currentPids);
    const newPids = currentPids.filter((pid) => !seenPids.has(pid));
    invalid(seenPids.size + newPids.length > MAX_TRACKED_HELPERS);
    for (const pid of newPids) seenPids.add(pid);
    const exitedPid = [...seenPids].sort((left, right) => left - right)
      .find((pid) => !current.has(pid));
    if (exitedPid !== undefined) return { exitedPid, initialPids: initial };
    if (Date.now() >= deadline) break;
    await delay(pollMs);
  } while (Date.now() <= deadline);
  throw new Error(FAILURE_MESSAGE);
}

function validateOptions({ platform, root, timeoutMs, pollMs, listProcesses }) {
  invalid(platform !== "darwin" && platform !== "win32");
  invalid(typeof root !== "string" || root.length === 0 || root.length > 1_024);
  invalid(!Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > MAX_WAIT_MS);
  invalid(!Number.isSafeInteger(pollMs) || pollMs < 1 || pollMs > MAX_POLL_MS);
  invalid(typeof listProcesses !== "function");
}

function validateInitialPids(pids) {
  invalid(!Array.isArray(pids) || pids.length === 0 || pids.length > MAX_TRACKED_HELPERS);
  invalid(pids.some((pid) => !Number.isSafeInteger(pid) || pid < 2 || pid > 0xffff_ffff));
  return [...new Set(pids)].sort((left, right) => left - right);
}

function invalid(condition) {
  if (condition) throw new Error(FAILURE_MESSAGE);
}

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
