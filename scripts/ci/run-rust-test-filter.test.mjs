import assert from "node:assert/strict";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import test from "node:test";

import {
  buildCargoCommands,
  countListedTests,
  isDirectExecution,
  runFilteredRustTests,
} from "./run-rust-test-filter.mjs";

const FILTER = "services::browser::cef_supervision::capability_tests";

test("direct execution compares canonical paths on Windows", () => {
  const script = resolve("scripts/ci/run-rust-test-filter.mjs");
  const argvPath = process.platform === "win32" ? script.toUpperCase() : script;

  assert.equal(isDirectExecution(pathToFileURL(script), argvPath), true);
});

test("the inventory command and execution use the same Rust filter", () => {
  const commands = buildCargoCommands({
    filter: FILTER,
    features: "windows-tests",
    testThreads: 1,
  });

  assert.deepEqual(commands.inventory, [
    "test", "--lib", "--features", "windows-tests", FILTER,
    "--", "--test-threads=1", "--list",
  ]);
  assert.deepEqual(commands.execute, [
    "test", "--lib", "--features", "windows-tests", FILTER,
    "--", "--test-threads=1",
  ]);
});

test("only actual test inventory lines are counted", () => {
  assert.equal(countListedTests("module::one: test\nmodule::two: test\n2 tests, 0 benchmarks\n"), 2);
  assert.equal(countListedTests("0 tests, 0 benchmarks\n"), 0);
});

test("a zero-match filter fails before cargo can report success", () => {
  const calls = [];
  const run = (_command, args) => {
    calls.push(args);
    return { status: 0, signal: null, stdout: "0 tests, 0 benchmarks\n" };
  };

  assert.throws(
    () => runFilteredRustTests({ filter: FILTER }, run),
    /Rust test filter failed/u,
  );
  assert.equal(calls.length, 1);
});

test("an inventory failure reports bounded cargo diagnostics", () => {
  const oversized = "x".repeat(40_000);
  const run = () => ({
    status: 101,
    signal: "SIGKILL",
    stdout: "inventory stdout",
    stderr: `linker failed\n${oversized}`,
  });

  assert.throws(
    () => runFilteredRustTests({ filter: FILTER }, run),
    (error) => {
      assert.match(error.message, /inventory/u);
      assert.match(error.message, /status=101/u);
      assert.match(error.message, /signal=SIGKILL/u);
      assert.match(error.message, /inventory stdout/u);
      assert.match(error.message, /linker failed/u);
      assert.ok(error.message.length < oversized.length);
      return true;
    },
  );
});

test("a non-empty inventory is executed without a shell", () => {
  const calls = [];
  const run = (_command, args, options) => {
    calls.push({ args, options });
    return calls.length === 1
      ? { status: 0, signal: null, stdout: `${FILTER}::one: test\n` }
      : { status: 0, signal: null, stdout: "" };
  };

  assert.equal(runFilteredRustTests({ filter: FILTER, exact: false }, run), 1);
  assert.equal(calls.length, 2);
  assert.equal(calls[0].options.shell, false);
  assert.equal(calls[1].options.stdio, "inherit");
});

test("unknown or unsafe CLI values are rejected", () => {
  assert.throws(
    () => buildCargoCommands({ filter: "module\nother" }),
    /Rust test filter failed/u,
  );
  assert.throws(
    () => buildCargoCommands({ filter: FILTER, features: "bad feature" }),
    /Rust test filter failed/u,
  );
});
