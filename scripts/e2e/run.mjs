import { mkdtemp, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { reportNativeDiagnostics } from "./native-diagnostics.mjs";
import {
  buildArguments,
  cleanupProfile,
  debugBinaryPath,
  E2E_BUILD_TIMEOUT_MS,
  E2E_JOURNEY_TIMEOUT_MS,
  e2eCargoTargetDir,
  isAllowedProfilePath,
  runCommand,
} from "./e2e-process.mjs";

const repoRoot = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const profilePath = await realpath(await mkdtemp(join(tmpdir(), "beaver-e2e-")));
const canonicalTemp = await realpath(tmpdir());
const logDirectory = join(profilePath, "logs");
const cargoTargetDir = e2eCargoTargetDir(
  process.platform,
  repoRoot,
  process.env.CARGO_TARGET_DIR,
);

if (!isAllowedProfilePath(profilePath, canonicalTemp)) {
  throw new Error("E2E profile isolation failed");
}

const environment = {
  ...process.env,
  CARGO_TARGET_DIR: cargoTargetDir,
  CL_GO_CEF_TEST_DATA_DIR: profilePath,
  CLGO_CEF_DEV_PREP: "1",
  CLGO_CEF_CARGO_FEATURES: "e2e",
  E2E_APP_BINARY: debugBinaryPath(process.platform, cargoTargetDir),
  E2E_LOG_DIR: logDirectory,
  VITE_E2E: "1",
};

let hadPriorFailure = false;
try {
  if (process.env.E2E_SKIP_BUILD !== "1") {
    const buildExit = await runCommand(
      process.execPath,
      [resolve(repoRoot, "scripts/cef/run-tauri.mjs"), ...buildArguments(process.platform)],
      { cwd: repoRoot, env: environment, timeoutMs: E2E_BUILD_TIMEOUT_MS },
    );
    if (buildExit !== 0) process.exitCode = buildExit;
  }

  if (!process.exitCode) {
    process.exitCode = await runCommand(
      process.execPath,
      [resolve(repoRoot, "node_modules/@wdio/cli/bin/wdio.js"), "run", "wdio.conf.ts"],
      { cwd: repoRoot, env: environment, timeoutMs: E2E_JOURNEY_TIMEOUT_MS },
    );
  }
} catch (error) {
  hadPriorFailure = true;
  throw error;
} finally {
  // Every native journey owns the same bounded logs, so every failure must expose them
  // before the isolated profile is removed.
  if (hadPriorFailure || process.exitCode) {
    try {
      await reportNativeDiagnostics(logDirectory);
    } catch {
      process.stderr.write("Native diagnostic collection failed.\n");
    }
  }
  await cleanupProfile(profilePath, {
    tempPath: canonicalTemp,
    hadPriorFailure: hadPriorFailure || Boolean(process.exitCode),
  });
}
