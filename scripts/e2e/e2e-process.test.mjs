import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  buildArguments,
  debugBinaryPath,
  isAllowedProfilePath,
} from "./e2e-process.mjs";

const ciSource = readFileSync(new URL("../../.github/workflows/ci.yml", import.meta.url), "utf8");
const mainSource = readFileSync(new URL("../../src-tauri/src/main.rs", import.meta.url), "utf8");
const runnerSource = readFileSync(new URL("./run.mjs", import.meta.url), "utf8");

test("the E2E build always enables the isolated feature", () => {
  assert.deepEqual(buildArguments("linux"), [
    "build", "--debug", "--features", "e2e", "--config",
    "src-tauri/tauri.e2e.conf.json", "--no-bundle",
  ]);
  assert.deepEqual(buildArguments("darwin"), [
    "build", "--debug", "--features", "e2e", "--config",
    "src-tauri/tauri.e2e.conf.json", "--bundles", "app",
  ]);
});

test("the E2E binary path is platform specific", () => {
  assert.equal(debugBinaryPath("linux", "/repo"), "/repo/src-tauri/target/e2e/debug/cl-go-dash");
  assert.equal(debugBinaryPath("win32", "/repo"), "/repo/src-tauri/target/e2e/debug/cl-go-dash.exe");
  assert.equal(
    debugBinaryPath("darwin", "/repo"),
    "/repo/src-tauri/target/e2e/debug/bundle/macos/Beaver.app/Contents/MacOS/cl-go-dash",
  );
});

test("only a dedicated direct child of the system temp directory is accepted", () => {
  assert.equal(isAllowedProfilePath("/tmp/beaver-e2e-Ab12", "/tmp"), true);
  assert.equal(isAllowedProfilePath("/tmp/beaver-e2e-Ab12/nested", "/tmp"), false);
  assert.equal(isAllowedProfilePath("/tmp/another-profile", "/tmp"), false);
  assert.equal(isAllowedProfilePath("/repo", "/tmp"), false);
});

test("CI exercises Rust assertions and clippy with the E2E feature", () => {
  assert.match(ciSource, /cargo clippy --all-targets --features e2e -- -D warnings/u);
  assert.match(ciSource, /cargo test --all --features e2e/u);
});

test("release builds reject the E2E control feature in the application binary", () => {
  assert.match(
    mainSource,
    /cfg\(all\(feature = "e2e", not\(debug_assertions\)\)\)[\s\S]*compile_error!/u,
  );
});

test("profile cleanup cannot hide the preceding E2E failure", () => {
  assert.match(runnerSource, /rm\(profilePath, \{ recursive: true, force: true,/u);
});
