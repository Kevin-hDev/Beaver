import { mkdtemp, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { reportNativeDiagnostics } from "./native-diagnostics.mjs";
import { preparePackagedApp } from "./packaged-app.mjs";
import {
  buildArguments,
  canonicalE2eRepoRoot,
  cleanupProfile,
  debugBinaryPath,
  E2E_BUILD_TIMEOUT_MS,
  E2E_JOURNEY_TIMEOUT_MS,
  e2eCargoTargetDir,
  isAllowedProfilePath,
  runCommand,
} from "./e2e-process.mjs";

const repoRoot = await canonicalE2eRepoRoot(import.meta.url);
const packaged = process.env.E2E_PACKAGED === "1";
if (process.env.E2E_PACKAGED !== undefined && !packaged) {
  throw new Error("E2E packaged mode is invalid");
}
if (packaged && process.platform !== "win32") {
  throw new Error("E2E packaged mode is unsupported");
}
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
let packagedApp;
try {
  if (process.env.E2E_SKIP_BUILD !== "1") {
    const buildExit = await runCommand(
      process.execPath,
      [
        resolve(repoRoot, "scripts/cef/run-tauri.mjs"),
        ...buildArguments(process.platform, packaged),
      ],
      { cwd: repoRoot, env: environment, timeoutMs: E2E_BUILD_TIMEOUT_MS },
    );
    if (buildExit !== 0) process.exitCode = buildExit;
  }

  if (!process.exitCode) {
    if (packaged) {
      packagedApp = await preparePackagedApp({
        platform: process.platform,
        cargoTargetDir,
        profilePath,
        run: (command, args, options) => runCommand(
          command,
          args,
          {
            cwd: repoRoot,
            env: environment,
            timeoutMs: E2E_JOURNEY_TIMEOUT_MS,
            ...options,
          },
        ),
      });
      environment.E2E_APP_BINARY = packagedApp.binaryPath;
    }
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
  if (packagedApp) {
    try {
      await packagedApp.cleanup();
    } catch {
      process.stderr.write("Packaged E2E cleanup failed.\n");
      process.exitCode = 1;
    }
  }
  await cleanupProfile(profilePath, {
    tempPath: canonicalTemp,
    hadPriorFailure: hadPriorFailure || Boolean(process.exitCode),
  });
}
