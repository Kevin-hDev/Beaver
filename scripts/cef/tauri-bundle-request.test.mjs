import assert from "node:assert/strict";
import test from "node:test";

import { resolveWindowsBundleRequest } from "./tauri-bundle-request.mjs";

test("Windows build defaults to one explicit NSIS bundle", () => {
  assert.deepEqual(
    resolveWindowsBundleRequest({
      args: ["build", "--target", "x86_64-pc-windows-msvc"],
      platform: "win32",
    }),
    {
      args: ["build", "--target", "x86_64-pc-windows-msvc", "--bundles=nsis"],
      bundleType: "nsis",
    },
  );
});

test("Windows build normalizes one explicit supported bundle", () => {
  for (const [args, bundleType] of [
    [["build", "--bundles", "nsis"], "nsis"],
    [["build", "--bundles=msi"], "msi"],
    [["build", "-b", "msi"], "msi"],
  ]) {
    assert.deepEqual(resolveWindowsBundleRequest({ args, platform: "win32" }), {
      args: ["build", `--bundles=${bundleType}`],
      bundleType,
    });
  }
});

test("Windows build refuses ambiguous or unsupported bundle requests", () => {
  for (const args of [
    ["build", "--bundles", "nsis", "msi"],
    ["build", "--bundles", "nsis,msi"],
    ["build", "--bundles=nsis,"],
    ["build", "--bundles", "app"],
    ["build", "--bundles"],
    ["build", "--bundles", "nsis", "--no-bundle"],
  ]) {
    assert.throws(
      () => resolveWindowsBundleRequest({ args, platform: "win32" }),
      /invalid/u,
    );
  }
});

test("no-bundle and non-Windows launches do not invent a bundle type", () => {
  assert.deepEqual(
    resolveWindowsBundleRequest({
      args: ["build", "--no-bundle"],
      platform: "win32",
    }),
    { args: ["build", "--no-bundle"], bundleType: null },
  );
  assert.deepEqual(
    resolveWindowsBundleRequest({ args: ["build"], platform: "darwin" }),
    { args: ["build"], bundleType: null },
  );
  assert.deepEqual(
    resolveWindowsBundleRequest({ args: ["dev"], platform: "win32" }),
    { args: ["dev"], bundleType: null },
  );
});

test("the default bundle option is inserted before runner arguments", () => {
  assert.deepEqual(
    resolveWindowsBundleRequest({
      args: ["build", "--", "--application-argument"],
      platform: "win32",
    }),
    {
      args: ["build", "--bundles=nsis", "--", "--application-argument"],
      bundleType: "nsis",
    },
  );
});
