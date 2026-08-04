import assert from "node:assert/strict";
import test from "node:test";
import { normalizeNewlines } from "./text-contracts.mjs";

test("normalise LF, CRLF et CR sans modifier le contenu", () => {
  assert.equal(normalizeNewlines("a\nb\n"), "a\nb\n");
  assert.equal(normalizeNewlines("a\r\nb\r\n"), "a\nb\n");
  assert.equal(normalizeNewlines("a\rb\r"), "a\nb\n");
});

test("refuse une valeur non textuelle ou démesurée", () => {
  assert.throws(() => normalizeNewlines(null));
  assert.throws(() => normalizeNewlines(42));
  assert.throws(() => normalizeNewlines("x".repeat(2 * 1024 * 1024 + 1)));
});
