import { randomUUID } from "node:crypto";
import { mkdir, open, rename, rm, writeFile } from "node:fs/promises";
import { isAbsolute, join } from "node:path";
import { observeOwnedCefHelperTurnover } from "./native-cef-liveness-observer.mjs";

const PROOF_FILE = "cef-helper-turnover.json";
const MAX_PATH_CHARS = 1_024;
const MAX_PROOF_BYTES = 4_096;
const MAX_WAIT_MS = 30_000;
const MAX_POLL_MS = 1_000;
const FAILURE_MESSAGE = "Native CEF turnover proof failed";
const TIMESTAMP_PATTERN = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$/u;

export async function captureMacCefTurnoverProof({
  logDirectory,
  root,
  timeoutMs,
  observeTurnover = observeOwnedCefHelperTurnover,
}) {
  validatePath(logDirectory);
  validatePath(root);
  validateTimeout(timeoutMs);
  invalid(typeof observeTurnover !== "function");
  let payload;
  try {
    const result = await observeTurnover({
      platform: "darwin",
      root,
      timeoutMs,
    });
    validateResult(result);
    payload = { state: "completed", observedAt: new Date().toISOString(), ...result };
  } catch {
    payload = { state: "failed", observedAt: new Date().toISOString() };
  }
  await publishProof(logDirectory, payload);
}

export async function waitForMacCefTurnoverProof({
  logDirectory,
  timeoutMs,
  pollMs = 25,
}) {
  validatePath(logDirectory);
  validateTimeout(timeoutMs);
  invalid(!Number.isSafeInteger(pollMs) || pollMs < 1 || pollMs > MAX_POLL_MS);
  const path = join(logDirectory, PROOF_FILE);
  const deadline = Date.now() + timeoutMs;
  do {
    const proof = await readProofIfPresent(path);
    if (proof?.state === "failed") throw new Error(FAILURE_MESSAGE);
    if (proof?.state === "completed") {
      validateResult(proof);
      invalid(typeof proof.observedAt !== "string"
        || !TIMESTAMP_PATTERN.test(proof.observedAt));
      return {
        exitedPid: proof.exitedPid,
        initialPids: proof.initialPids,
        observedAt: proof.observedAt,
      };
    }
    if (Date.now() >= deadline) break;
    await new Promise((resolve) => setTimeout(resolve, pollMs));
  } while (Date.now() <= deadline);
  throw new Error(FAILURE_MESSAGE);
}

async function publishProof(logDirectory, payload) {
  await mkdir(logDirectory, { recursive: true, mode: 0o700 });
  const path = join(logDirectory, PROOF_FILE);
  const temporary = `${path}.${randomUUID()}.tmp`;
  try {
    await writeFile(temporary, `${JSON.stringify(payload)}\n`, { flag: "wx", mode: 0o600 });
    await rename(temporary, path);
  } finally {
    await rm(temporary, { force: true });
  }
}

async function readProofIfPresent(path) {
  let handle;
  try {
    handle = await open(path, "r");
    const stats = await handle.stat();
    invalid(!stats.isFile() || stats.size < 1 || stats.size > MAX_PROOF_BYTES);
    const buffer = Buffer.alloc(stats.size);
    const { bytesRead } = await handle.read(buffer, 0, stats.size, 0);
    invalid(bytesRead !== stats.size);
    return JSON.parse(buffer.toString("utf8"));
  } catch (error) {
    if (error?.code === "ENOENT") return undefined;
    throw new Error(FAILURE_MESSAGE);
  } finally {
    if (handle) await handle.close().catch(() => undefined);
  }
}

function validateResult(result) {
  invalid(!result || typeof result !== "object");
  invalid(!Number.isSafeInteger(result.exitedPid)
    || result.exitedPid < 2 || result.exitedPid > 0xffff_ffff);
  invalid(!Array.isArray(result.initialPids)
    || result.initialPids.length === 0 || result.initialPids.length > 64);
  invalid(new Set(result.initialPids).size !== result.initialPids.length);
  invalid(result.initialPids.some((pid) => !Number.isSafeInteger(pid)
    || pid < 2 || pid > 0xffff_ffff));
}

function validatePath(value) {
  invalid(typeof value !== "string" || value.length === 0
    || value.length > MAX_PATH_CHARS || !isAbsolute(value));
}

function validateTimeout(value) {
  invalid(!Number.isSafeInteger(value) || value < 1 || value > MAX_WAIT_MS);
}

function invalid(condition) {
  if (condition) throw new Error(FAILURE_MESSAGE);
}
