import assert from "node:assert/strict";
import { test } from "node:test";

import { snapshotToolResult } from "../../src-tauri/resources/extension-host/tool-result-snapshot.mjs";
import { boundedToolResult } from "../../src-tauri/resources/extension-host/loader.mjs";
import { LIMITS } from "../../src-tauri/resources/extension-host/contract.mjs";
import { encodeProtocolMessage } from "../../src-tauri/resources/extension-host/protocol-output.mjs";

test("le bornage final conserve une copie gelée sans prototype", () => {
  const result = boundedToolResult(snapshotToolResult({
    content: [{ type: "text", text: "stable" }],
    isError: false,
  }));
  assert.equal(Object.getPrototypeOf(result), null);
  assert.equal(Object.isFrozen(result), true);
  assert.equal(Object.getPrototypeOf(result.content[0]), null);
  assert.equal(Object.isFrozen(result.content), true);
  assert.throws(() => { result.content[0].text = "changed"; }, TypeError);
  assert.equal(result.content[0].text, "stable");
});

test("scripts/extensions/tool-result-snapshot.test.mjs captures every result, array and block field once", () => {
  const legacy = snapshotToolResult("legacy");
  assert.equal(legacy.content, "legacy");
  assert.equal(legacy.isError, false);
  const reads = new Map();
  const count = (name, value) => () => {
    reads.set(name, (reads.get(name) ?? 0) + 1);
    return value;
  };
  const text = new Proxy({}, { get: (_target, property) => ({
    type: count("text.type", "text"), text: count("text.text", "one"),
  })[property]?.() });
  const file = new Proxy({}, { get: (_target, property) => ({
    type: count("file.type", "file"), path: count("file.path", "out.png"),
    purpose: count("file.purpose", "preview"), displayName: count("file.displayName", "Output"),
  })[property]?.() });
  const blocks = new Proxy([text, file], { get: (target, property, receiver) => {
    if (property === "length" || property === "0" || property === "1") {
      reads.set(`blocks.${String(property)}`, (reads.get(`blocks.${String(property)}`) ?? 0) + 1);
    }
    return Reflect.get(target, property, receiver);
  } });
  const result = new Proxy({}, { get: (_target, property) => ({
    content: count("result.content", blocks), isError: count("result.isError", false),
    displaySummary: count("result.displaySummary", undefined), truncated: count("result.truncated", false),
  })[property]?.() });
  const snapshot = snapshotToolResult(result);
  blocks[0] = { type: "text", text: "mutated" };

  for (const value of reads.values()) assert.equal(value, 1);
  assert.equal(snapshot.content[0].text, "one");
  assert.equal(snapshot.content.length, 2);
});

test("scripts/extensions/tool-result-snapshot.test.mjs accepts exact limits and rejects the next item", () => {
  const exact = [
    ...Array.from({ length: LIMITS.maxResultFiles }, (_, index) => ({ type: "file", path: `out-${index}.txt`, purpose: "artifact" })),
    ...Array.from({ length: LIMITS.maxResultBlocks - LIMITS.maxResultFiles }, () => ({ type: "text", text: "x" })),
  ];
  assert.equal(snapshotToolResult({ content: exact }).content.length, LIMITS.maxResultBlocks);
  assert.throws(() => snapshotToolResult({ content: [...exact, { type: "text", text: "x" }] }));
  assert.throws(() => snapshotToolResult({ content: Array.from({ length: LIMITS.maxResultFiles + 1 }, () => ({ type: "file", path: "x", purpose: "artifact" })) }));
});

test("scripts/extensions/tool-result-snapshot.test.mjs rejects empty or control display names", () => {
  for (const displayName of ["", "line\nbreak"]) {
    assert.throws(() => snapshotToolResult({
      content: [{ type: "file", path: "out.png", purpose: "preview", displayName }],
    }));
  }
});

test("scripts/extensions/tool-result-snapshot.test.mjs validates purposes without Array prototype dispatch", () => {
  const original = Array.prototype.includes;
  Array.prototype.includes = () => { throw new Error("poisoned prototype"); };
  try {
    const result = snapshotToolResult({
      content: [{ type: "file", path: "out.png", purpose: "preview" }],
    });
    assert.equal(result.content[0].purpose, "preview");
  } finally {
    Array.prototype.includes = original;
  }
});

test("scripts/extensions/tool-result-snapshot.test.mjs measures escaped final JSON", () => {
  const result = snapshotToolResult({
    content: [{ type: "text", text: "\\".repeat(LIMITS.maxResultTextBytes) }],
  });
  assert.throws(() => encodeProtocolMessage({ jsonrpc: "2.0", id: 1, result }));
});

test("scripts/extensions/tool-result-snapshot.test.mjs refuses excess blocks and files", () => {
  assert.throws(() => snapshotToolResult({ content: Array.from({ length: 17 }, () => ({ type: "text", text: "x" })) }));
  assert.throws(() => snapshotToolResult({ content: Array.from({ length: 9 }, () => ({ type: "file", path: "x", purpose: "artifact" })) }));
});

test("scripts/extensions/tool-result-snapshot.test.mjs preserves an immutable no-prototype snapshot", () => {
  const source = { content: [{ type: "text", text: "before" }], ignored: "secret" };
  const snapshot = snapshotToolResult(source);
  source.content[0].text = "after";
  assert.equal(Object.getPrototypeOf(snapshot), null);
  assert.equal(snapshot.content[0].text, "before");
  assert.equal("ignored" in snapshot, false);
});

test("scripts/extensions/tool-result-snapshot.test.mjs bounds Unicode text by UTF-8 bytes", () => {
  assert.throws(() => snapshotToolResult({ content: [{ type: "text", text: "🦫".repeat(Math.floor(LIMITS.maxResultTextBytes / Buffer.byteLength("🦫", "utf8")) + 1) }] }));
});

test("scripts/extensions/tool-result-snapshot.test.mjs encodes rich content and rejects an oversized array", () => {
  const message = { jsonrpc: "2.0", id: 1, result: snapshotToolResult({ content: [{ type: "text", text: "😀 \\\"" }] }) };
  assert.ok(encodeProtocolMessage(message).length < LIMITS.maxMessageBytes);
  assert.throws(() => encodeProtocolMessage({ jsonrpc: "2.0", id: 1, result: { content: [{ type: "text", text: "x".repeat(LIMITS.maxMessageBytes) }] } }));
});
