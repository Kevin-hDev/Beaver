import assert from "node:assert/strict";
import { test } from "node:test";
import {
  parseChangedRustFiles,
  rustfmtArguments,
  validateBaseRevision,
} from "./check-rust-format.mjs";

test("changed Rust files are limited to backend source files", () => {
  const changed = [
    "src-tauri/src/services/agent_local/session_store_updates.rs",
    "src-tauri/src/lib.rs",
    "src-tauri/tests/example.rs",
    "scripts/example.rs",
    "src-tauri/src/not-rust.txt",
  ].join("\0");

  assert.deepEqual(parseChangedRustFiles(`${changed}\0`), [
    "src-tauri/src/services/agent_local/session_store_updates.rs",
    "src-tauri/src/lib.rs",
  ]);
});

test("changed Rust file collection is bounded", () => {
  const changed = Array.from(
    { length: 4_097 },
    (_, index) => `src-tauri/src/generated/file-${index}.rs`,
  ).join("\0");

  assert.throws(() => parseChangedRustFiles(`${changed}\0`), /too many changed files/u);
});

test("oversized Git output and paths fail before collection processing", () => {
  assert.throws(
    () => parseChangedRustFiles("x".repeat(1024 * 1024 + 1)),
    /Git output is too large/u,
  );
  assert.throws(
    () => parseChangedRustFiles(`src-tauri/src/${"x".repeat(513)}.rs\0`),
    /invalid changed file/u,
  );
  assert.throws(
    () => parseChangedRustFiles("src-tauri/src/../outside.rs\0"),
    /invalid changed file/u,
  );
});

test("base revisions and rustfmt arguments cannot become shell input", () => {
  assert.equal(validateBaseRevision("a".repeat(40)), "a".repeat(40));
  assert.throws(() => validateBaseRevision("HEAD; remove-everything"), /invalid base revision/u);
  assert.deepEqual(rustfmtArguments("src-tauri/src/lib.rs"), [
    "--edition",
    "2021",
    "--check",
    "--config",
    "skip_children=true",
    "src-tauri/src/lib.rs",
  ]);
});
