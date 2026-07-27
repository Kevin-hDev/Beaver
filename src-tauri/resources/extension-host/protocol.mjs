import { randomUUID } from "node:crypto";

const MAX_MESSAGE_BYTES = 1_048_576;
const MAX_PENDING = 64;
const MAX_IN_FLIGHT_REQUESTS = 64;
const REQUEST_TIMEOUT_MS = 30_000;
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
  if (pending.size >= MAX_PENDING) {
    return Promise.reject(new Error("too_many_pending_requests"));
  }
  const id = randomUUID();
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error("core_request_timeout"));
    }, REQUEST_TIMEOUT_MS);
    timer.unref();
    pending.set(id, { resolve, reject, timer });
    try {
      send({ jsonrpc: "2.0", id, method, params });
    } catch (error) {
      clearTimeout(timer);
      pending.delete(id);
      reject(error);
    }
  });
}

function receiveChunk(chunk) {
  input = Buffer.concat([input, chunk]);
  if (input.length > MAX_MESSAGE_BYTES) {
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
  if (inFlightRequests >= MAX_IN_FLIGHT_REQUESTS) {
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
    request.reject(new Error("core_request_failed"));
  } else {
    request.resolve(message.result);
  }
}

function send(message) {
  const line = Buffer.from(`${JSON.stringify(message)}\n`, "utf8");
  if (line.length > MAX_MESSAGE_BYTES) {
    throw new Error("message_too_large");
  }
  writeProtocol(line);
}
