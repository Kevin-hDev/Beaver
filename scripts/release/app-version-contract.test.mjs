import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { assertAppVersionContract } from "./app-version-contract.mjs";

const valid = {
  packageJson: JSON.stringify({ version: "1.2.3" }),
  cargoToml: '[package]\nname = "cl-go-dash"\nversion = "1.2.3"\n',
  tauriConfig: JSON.stringify({ version: "1.2.3" }),
};

test("accepts one stable application version", () => {
  assert.doesNotThrow(() => assertAppVersionContract(valid));
});

test("the repository keeps all application versions aligned", () => {
  assert.doesNotThrow(() =>
    assertAppVersionContract({
      packageJson: readFileSync("package.json", "utf8"),
      cargoToml: readFileSync("src-tauri/Cargo.toml", "utf8"),
      tauriConfig: readFileSync("src-tauri/tauri.conf.json", "utf8"),
    }),
  );
});

test("rejects any disagreement between release manifests", () => {
  for (const field of ["packageJson", "cargoToml", "tauriConfig"]) {
    assert.throws(
      () =>
        assertAppVersionContract({
          ...valid,
          [field]: valid[field].replace("1.2.3", "1.2.4"),
        }),
      /Application version contract failed/u,
    );
  }
});

test("rejects prerelease and malformed versions", () => {
  for (const version of ["1.2", "1.2.3.4", "1.2.0-rc.1"]) {
    assert.throws(
      () =>
        assertAppVersionContract({
          ...valid,
          packageJson: valid.packageJson.replace("1.2.3", version),
        }),
      /Application version contract failed/u,
    );
  }
});

test("bounds parsed manifests and keeps the public error generic", () => {
  assert.throws(
    () => assertAppVersionContract({ ...valid, cargoToml: "x".repeat(2_097_153) }),
    (error) => error.message === "Application version contract failed",
  );
});
