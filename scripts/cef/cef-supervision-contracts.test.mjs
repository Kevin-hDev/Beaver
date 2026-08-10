import assert from "node:assert/strict";
import test from "node:test";

import {
  validateCefSupervisionContracts,
  validateRepository,
} from "./cef-supervision-contracts.mjs";

const valid = {
  workflow: `
backend-windows-native:
  run: cargo clippy --all-targets -- -D warnings
  run: cargo test --lib --features windows-tests services::browser::cef_supervision::windows_tracker_tests
backend-macos-native:
  run: cargo clippy --all-targets -- -D warnings
  run: cargo test --lib services::browser::cef_supervision::macos_tracker_tests
`,
  build: `if target == "macos" || (target == "windows" && !windows_tests) {
    println!("cargo:rustc-cfg=native_browser");
  }`,
  macHelper: "sandbox.initialize(args.as_main_args()); bootstrap.admit_after_sandbox()",
};

test("native CI proves both CEF supervision authorities without enabling Linux", () => {
  assert.deepEqual(validateCefSupervisionContracts(valid), []);
});

test("missing native tests and sandbox bypasses fail the contract", () => {
  const errors = validateCefSupervisionContracts({
    ...valid,
    workflow: valid.workflow
      .replace("services::browser::cef_supervision::macos_tracker_tests", "missing")
      .concat("\nCEF_NO_SANDBOX: 1\n"),
  });

  assert.ok(errors.includes("macOS supervision tests are missing"));
  assert.ok(errors.includes("CEF sandbox bypass is forbidden"));
});

test("Linux native_browser activation fails the contract", () => {
  const errors = validateCefSupervisionContracts({
    ...valid,
    build: `${valid.build}\nif target == "linux" { println!("cargo:rustc-cfg=native_browser"); }`,
  });

  assert.ok(errors.includes("Linux native_browser must remain disabled"));
});

test("the checked-in repository satisfies the native supervision contract", async () => {
  assert.deepEqual(await validateRepository(), []);
});
