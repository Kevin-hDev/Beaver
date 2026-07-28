import { LIMITS } from "./contract.mjs";

export function encodeProtocolMessage(message) {
  const fitted = fitToolResult(message);
  const line = Buffer.from(`${JSON.stringify(fitted)}\n`, "utf8");
  if (line.length >= LIMITS.maxMessageBytes) {
    throw new Error("message_too_large");
  }
  return line;
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
  let fitted = {
    ...message,
    result: { ...message.result, content: "", truncated: true },
  };
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
