import assert from "node:assert/strict";
import { test } from "node:test";
import { assertExactSuccess, contractArguments } from "./check.mjs";

test("Windows contracts use the native test profile without changing exact collection", () => {
  assert.deepEqual(contractArguments("wanted", "win32"), [
    "test", "--lib", "--features", "windows-tests", "wanted", "--", "--test-threads=1", "--exact",
  ]);
  for (const platform of ["linux", "darwin"]) {
    assert.deepEqual(contractArguments("wanted", platform), [
      "test", "--lib", "wanted", "--", "--test-threads=1", "--exact",
    ]);
  }
  assert.throws(() => contractArguments("--ignored", "win32"));
});

test("zero collected, ignored, renamed, failed and interrupted tests cannot pass", () => {
  for (const result of [
    { status: 0, stdout: "test result: ok. 0 passed; 0 failed; 0 ignored;" },
    { status: 0, stdout: "test wanted ... ignored\ntest result: ok. 0 passed; 0 failed; 1 ignored;" },
    { status: 0, stdout: "test other ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;" },
    { status: 1, stdout: "test wanted ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored;" },
    { status: null, error: new Error("interrupted") },
  ]) assert.throws(() => assertExactSuccess("wanted", result));
});

test("the exact executed passing test is accepted on Unix and Windows", () => {
  for (const newline of ["\n", "\r\n"]) {
    assert.doesNotThrow(() => assertExactSuccess("wanted", {
      status: 0, stdout: ["test wanted ... ok", "test result: ok. 1 passed; 0 failed; 0 ignored;"].join(newline),
    }));
  }
});
