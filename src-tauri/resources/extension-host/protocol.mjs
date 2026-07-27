import { randomUUID } from "node:crypto";
import { LIMITS } from "./contract.mjs";

const REQUEST_TIMEOUT_MS = 30_000;
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
let inFlightRequests = 0;

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
    }, REQUEST_TIMEOUT_MS);
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
  let message;
  try {
    message = JSON.parse(line.toString("utf8"));
  } catch {
    process.exit(1);
    return;
  }
  if (!message || message.jsonrpc !== "2.0" || typeof message.id !== "string") {
    process.exit(1);
    return;
  }
  if (typeof message.method !== "string") {
    settle(message);
    return;
  }
  if (inFlightRequests >= LIMITS.maxInFlightRequests) {
    send({
      jsonrpc: "2.0",
      id: message.id,
      error: { code: -32_000, message: "extension_host_busy" },
    });
    return;
  }
  inFlightRequests += 1;
  try {
    const result = await handler(message.method, message.params ?? {});
    send({ jsonrpc: "2.0", id: message.id, result });
  } catch {
    send({
      jsonrpc: "2.0",
      id: message.id,
      error: { code: -32_000, message: "extension_host_request_failed" },
    });
  } finally {
    inFlightRequests -= 1;
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
    typeof reason === "string" && ERROR_REASON_PATTERN.test(reason)
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
  const fitted = fitToolResult(message);
  const line = Buffer.from(`${JSON.stringify(fitted)}\n`, "utf8");
  if (line.length >= LIMITS.maxMessageBytes) {
    throw new Error("message_too_large");
  }
  writeProtocol(line);
}

function fitToolResult(message) {
  if (encodedLength(message) < LIMITS.maxMessageBytes) return message;
  if (
    !message?.result
    || typeof message.result !== "object"
    || typeof message.result.content !== "string"
  ) {
    return message;
  }
  const source = message.result.content;
  let minimum = 0;
  let maximum = source.length;
  let fitted = { ...message, result: { ...message.result, content: "", truncated: true } };
  while (minimum <= maximum) {
    const middle = Math.floor((minimum + maximum) / 2);
    const candidate = {
      ...fitted,
      result: { ...fitted.result, content: safeSlice(source, middle) },
    };
    if (encodedLength(candidate) < LIMITS.maxMessageBytes) {
      fitted = candidate;
      minimum = middle + 1;
    } else {
      maximum = middle - 1;
    }
  }
  return fitted;
}

function encodedLength(message) {
  return Buffer.byteLength(`${JSON.stringify(message)}\n`, "utf8");
}

function safeSlice(value, end) {
  const adjusted = end > 0
    && end < value.length
    && isHighSurrogate(value.charCodeAt(end - 1))
    && isLowSurrogate(value.charCodeAt(end))
    ? end - 1
    : end;
  return value.slice(0, adjusted);
}

function isHighSurrogate(value) {
  return value >= 0xd800 && value <= 0xdbff;
}

function isLowSurrogate(value) {
  return value >= 0xdc00 && value <= 0xdfff;
}
