import { describe, expect, it } from "vitest";
import { validateBranchName } from "./branch-name";

describe("validateBranchName", () => {
  it.each([
    ["foo..bar"],
    ["foo\0bar"],
    ["foo bar"],
    ["a".repeat(101)],
    ["foo\\bar"],
    ["foo:bar"],
    [".hidden"],
    ["foo/"],
    ["foo//bar"],
    ["foo~bar"],
    ["foo^bar"],
    ["foo?bar"],
    ["foo*bar"],
    ["foo[bar"],
    ["foo@{bar"],
    ["feature.lock"],
    ["foo\tbar"],
    ["foo\nbar"],
    ["foo\rbar"],
    ["foo\x7fbar"],
    ["/foo"],
    ["foo."],
    ["-feature"],
  ])("rejette %s", (name) => {
    expect(validateBranchName(name).valid).toBe(false);
  });

  it.each([
    ["feature"],
    ["feature/foo"],
    ["feature/foo/bar"],
    ["a".repeat(100)],
  ])("accepte %s", (name) => {
    expect(validateBranchName(name)).toEqual({ valid: true });
  });
});
