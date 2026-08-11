import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFile, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import test from "node:test";

import {
  isDirectExecution,
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
  macBootstrap: "libc::getppid(); start_monitor(parent_pid)",
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

test("macOS helper checks only its validated parent identity after sandbox", () => {
  const errors = validateCefSupervisionContracts({
    ...valid,
    macBootstrap: "parent_identity.is_alive(); start_monitor(parent_identity)",
  });

  assert.ok(errors.includes("macOS helper must compare its current parent after sandbox"));
  assert.ok(errors.includes("macOS helper must not inspect its parent after sandbox"));
});

test("the checked-in repository satisfies the native supervision contract", async () => {
  assert.deepEqual(await validateRepository(), []);
});

test("direct execution is detected from canonical filesystem paths", () => {
  const script = resolve("scripts/cef/cef-supervision-contracts.mjs");

  assert.equal(isDirectExecution(pathToFileURL(script), script), true);
  assert.equal(isDirectExecution(pathToFileURL(script), `${script}.other`), false);
  assert.equal(isDirectExecution(pathToFileURL(script), undefined), false);
});

test("an invalid fixture fails when the contract is executed directly", async () => {
  const fixture = await mkdtemp(join(tmpdir(), "beaver-cef-contract-"));
  const scriptDirectory = join(fixture, "scripts", "cef");
  const workflowDirectory = join(fixture, ".github", "workflows");
  const browserDirectory = join(fixture, "src-tauri", "src", "services", "browser");
  const macSupervisionDirectory = join(browserDirectory, "cef_supervision", "macos");
  try {
    await Promise.all([
      mkdir(scriptDirectory, { recursive: true }),
      mkdir(workflowDirectory, { recursive: true }),
      mkdir(browserDirectory, { recursive: true }),
      mkdir(macSupervisionDirectory, { recursive: true }),
    ]);
    const copiedScript = join(scriptDirectory, "cef-supervision-contracts.mjs");
    await Promise.all([
      copyFile(resolve("scripts/cef/cef-supervision-contracts.mjs"), copiedScript),
      copyFile(
        resolve("scripts/cef/direct-execution.mjs"),
        join(scriptDirectory, "direct-execution.mjs"),
      ),
      writeFile(join(workflowDirectory, "ci.yml"), "jobs: {}\n", "utf8"),
      writeFile(join(fixture, "src-tauri", "build.rs"), "fn main() {}\n", "utf8"),
      writeFile(join(browserDirectory, "macos_helper_entry.rs"), "fn run() {}\n", "utf8"),
      writeFile(join(macSupervisionDirectory, "bootstrap.rs"), "fn run() {}\n", "utf8"),
    ]);

    const result = spawnSync(process.execPath, [copiedScript], {
      cwd: fixture,
      encoding: "utf8",
      shell: false,
    });

    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /native job is missing/u);
  } finally {
    await rm(fixture, { recursive: true, force: true });
  }
});
