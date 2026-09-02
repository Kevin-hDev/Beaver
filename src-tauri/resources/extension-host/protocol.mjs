import { randomUUID } from "node:crypto";
import {
  LIMITS,
  methodKind,
  PROTOCOL_ERROR_REASONS,
  TIMEOUTS,
} from "./contract.mjs";
import { encodeProtocolMessage } from "./protocol-output.mjs";

const MAX_REQUEST_ID_CHARS = 128;
const ERROR_REASON_PATTERN = /^[a-z][a-z0-9_]{0,63}$/;
const RETRYABLE_REASONS = new Set([
  "core_busy",
  "core_request_timeout",
  "core_transport_failed",
]);
// Capture the protocol writer before host.mjs silences accidental stdout writes.
const writeProtocol = process.stdout.write.bind(process.stdout);
const pending = new Map();
let input = Buffer.alloc(0);
let handler;
let fatal = false;
const activeRequestIds = new Set();

export function startProtocol(requestHandler) {
  handler = requestHandler;
  process.stdin.on("data", receiveChunk);
  process.stdin.on("end", () => process.exit(0));
  process.stdin.resume();
}

export function callCore(method, params = {}) {
  if (pending.size >= LIMITS.maxPendingRequests) {
    return Promise.reject(coreError(-32_000, "core_busy"));
  }
  const id = randomUUID();
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(coreError(-32_000, "core_request_timeout"));
    }, TIMEOUTS.coreRequestTimeoutMs);
    timer.unref();
    pending.set(id, { resolve, reject, timer });
    try {
      send({ jsonrpc: "2.0", id, method, params });
    } catch {
      clearTimeout(timer);
      pending.delete(id);
      reject(coreError(-32_000, "core_transport_failed"));
    }
  });
}

export function notifyCore(method, params = {}) {
  if (methodKind(method) !== "notification") {
    throw new Error("core_method_unavailable");
  }
  send({ jsonrpc: "2.0", method, params });
}

export function fatalProtocolExit() {
  if (fatal) return;
  fatal = true;
  process.stdin.pause();
  let responses;
  try {
    responses = [...activeRequestIds].map((id) =>
      encodeProtocolMessage({
        jsonrpc: "2.0",
        id,
        error: { code: -32_000, message: "extension_host_fatal" },
      }));
  } catch {
    process.exit(1);
    return;
  }
  if (responses.length === 0) {
    process.exit(1);
    return;
  }
  const timeout = setTimeout(() => process.exit(1), 100);
  try {
    writeProtocol(Buffer.concat(responses), () => {
      clearTimeout(timeout);
      process.exit(1);
    });
  } catch {
    clearTimeout(timeout);
    process.exit(1);
  }
}

function receiveChunk(chunk) {
  input = Buffer.concat([input, chunk]);
  if (input.length > LIMITS.maxMessageBytes) {
    process.exit(1);
  }
  let newline;
  while ((newline = input.indexOf(10)) !== -1) {
    const line = input.subarray(0, newline);
    input = input.subarray(newline + 1);
    if (line.length > 0) void receiveLine(line);
  }
}

async function receiveLine(line) {
  if (fatal) return;
  let message;
  try {
    message = JSON.parse(line.toString("utf8"));
  } catch {
    process.exit(1);
    return;
  }
  if (
    !message
    || message.jsonrpc !== "2.0"
    || typeof message.id !== "string"
    || message.id.length < 1
    || message.id.length > MAX_REQUEST_ID_CHARS
  ) {
    process.exit(1);
    return;
  }
  if (typeof message.method !== "string") {
    settle(message);
    return;
  }
  if (
    activeRequestIds.size >= LIMITS.maxInFlightRequests
    || activeRequestIds.has(message.id)
  ) {
    send({
      jsonrpc: "2.0",
      id: message.id,
      error: { code: -32_000, message: "extension_host_busy" },
    });
    return;
  }
  activeRequestIds.add(message.id);
  try {
    const result = await handler(message.method, message.params ?? {});
    if (!fatal) send({ jsonrpc: "2.0", id: message.id, result });
  } catch {
    if (!fatal) {
      send({
        jsonrpc: "2.0",
        id: message.id,
        error: { code: -32_000, message: "extension_host_request_failed" },
      });
    }
  } finally {
    activeRequestIds.delete(message.id);
  }
}

function settle(message) {
  const request = pending.get(message.id);
  if (!request) return;
  clearTimeout(request.timer);
  pending.delete(message.id);
  if (message.error) {
    request.reject(coreError(message.error.code, message.error.message));
  } else {
    request.resolve(message.result);
  }
}

function coreError(code, reason) {
  const safeCode =
    Number.isSafeInteger(code) && code >= -32_768 && code <= -32_000
      ? code
      : -32_000;
  const safeReason =
    typeof reason === "string"
      && ERROR_REASON_PATTERN.test(reason)
      && PROTOCOL_ERROR_REASONS.includes(reason)
      ? reason
      : "core_request_failed";
  const error = new Error(safeReason);
  error.name = "BeaverExtensionError";
  error.code = safeCode;
  error.reason = safeReason;
  error.retryable = RETRYABLE_REASONS.has(safeReason);
  return error;
}

function send(message) {
  writeProtocol(encodeProtocolMessage(message));
}
