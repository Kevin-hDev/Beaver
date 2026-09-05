import { createHash } from "node:crypto";
import { copyFile, lstat, mkdir, mkdtemp, readFile, realpath } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { reportNativeDiagnostics } from "./native-diagnostics.mjs";
import { persistNativeDiagnostics } from "./native-diagnostics-artifact.mjs";
import { prepareE2eArtifactDirectory } from "./e2e-artifact-directory.mjs";
import { preparePackagedApp } from "./packaged-app.mjs";
import {
  buildArguments,
  canonicalE2eRepoRoot,
  cleanupProfile,
  debugBinaryPath,
  diagnosticExtensionHostRoot,
  E2E_BUILD_TIMEOUT_MS,
  E2E_JOURNEY_TIMEOUT_MS,
  e2eCargoTargetDir,
  isAllowedProfilePath,
  resolveE2eRunMode,
  runCommand,
} from "./e2e-process.mjs";

const repoRoot = await canonicalE2eRepoRoot(import.meta.url);
const artifactDirectory = await prepareE2eArtifactDirectory(
  repoRoot,
  process.env.E2E_ARTIFACT_DIR,
);
const packaged = process.env.E2E_PACKAGED === "1";
if (process.env.E2E_PACKAGED !== undefined && !packaged) {
  throw new Error("E2E packaged mode is invalid");
}
const runMode = resolveE2eRunMode(process.env);
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
if (artifactDirectory) environment.E2E_ARTIFACT_DIR = artifactDirectory;
environment.BEAVER_E2E_UI_MANIFEST_SHA = await prepareUiRuntimeProof(repoRoot, profilePath);
if (packaged && process.platform === "linux") {
  environment.APPIMAGE_EXTRACT_AND_RUN = "1";
}

let hadPriorFailure = false;
let packagedApp;
try {
  if (runMode.build) {
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

  if (!process.exitCode && runMode.journey) {
    if (packaged) {
      packagedApp = await preparePackagedApp({
        platform: process.platform,
        cargoTargetDir,
        profilePath,
        diagnosticExtensionHostRoot: diagnosticExtensionHostRoot(
          process.env,
          repoRoot,
        ),
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
  if (artifactDirectory) {
    try {
      await persistNativeDiagnostics(logDirectory, artifactDirectory);
    } catch {
      process.stderr.write("Native diagnostic artifact failed.\n");
      process.exitCode = 1;
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

async function prepareUiRuntimeProof(root, profile) {
  const source = resolve(root, "src-tauri/tests/fixtures/extensions/ui-advanced/entry.mjs");
  const [metadata, canonical] = await Promise.all([lstat(source), realpath(source)]);
  if (
    !metadata.isFile()
    || metadata.isSymbolicLink()
    || canonical !== source
    || metadata.size > 65_536
  ) {
    throw new Error("E2E UI proof preparation failed");
  }
  const bytes = await readFile(canonical);
  const hash = createHash("sha256").update(bytes).digest("hex");
  const destination = join(profile, "extensions-ui-proof", "ui-proof", hash);
  await mkdir(destination, { recursive: true, mode: 0o700 });
  await copyFile(canonical, join(destination, "entry.mjs"));
  return hash;
}
