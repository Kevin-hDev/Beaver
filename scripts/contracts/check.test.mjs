import assert from "node:assert/strict";
import { test } from "node:test";
import { assertExactSuccess } from "./check.mjs";

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
