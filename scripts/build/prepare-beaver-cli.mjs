import { join } from "node:path";

import { runCommand } from "./command-runner.mjs";
import { normalizeCargoTargetDir } from "./cargo-target-dir.mjs";
import {
  canonicalDirectory,
  copyVerifiedAtomic,
  MAX_HELPER_BYTES,
  validateAbsolutePath,
} from "./updater-helper-copy.mjs";

const ERROR_MESSAGE = "Beaver CLI preparation failed";
const TARGET_PATTERN = /^[A-Za-z0-9_.-]{1,128}$/u;
// Tauri resolves external binaries from this exact target-suffixed filename.
const HOST_TARGETS = new Map([
  ["darwin:arm64", "aarch64-apple-darwin"],
  ["darwin:x64", "x86_64-apple-darwin"],
  ["linux:x64", "x86_64-unknown-linux-gnu"],
  ["win32:x64", "x86_64-pc-windows-msvc"],
]);

function fail() {
  throw new Error(ERROR_MESSAGE);
}

export function createBeaverCliBuildPlan({
  platform,
  arch = process.arch,
  target = "",
  tauriDir,
  cargoTargetDir = "",
} = {}) {
  try {
    const effectiveTarget = target || HOST_TARGETS.get(`${platform}:${arch}`) || "";
    if (!TARGET_PATTERN.test(effectiveTarget)) fail();
    validateAbsolutePath(tauriDir);
    const cargoRoot = cargoTargetDir
      ? normalizeCargoTargetDir(cargoTargetDir)
      : join(tauriDir, "target");
    const cargoArgs = ["build", "--release", "--bin", "beaver"];
    if (target) cargoArgs.push("--target", target);
    const windows = effectiveTarget.includes("-windows-");
    const executable = `beaver${windows ? ".exe" : ""}`;
    const targetRoot = target ? join(cargoRoot, target) : cargoRoot;
    return {
      cargoArgs,
      source: join(targetRoot, "release", executable),
      destination: join(
        tauriDir,
        "target",
        "beaver-cli",
        `beaver-${effectiveTarget}${windows ? ".exe" : ""}`,
      ),
    };
  } catch {
    fail();
  }
}

export async function prepareBeaverCli({
  platform,
  arch = process.arch,
  target = "",
  tauriDir,
  cargoTargetDir = "",
  run = runCommand,
} = {}) {
  try {
    if (typeof run !== "function") fail();
    const plan = createBeaverCliBuildPlan({ platform, arch, target, tauriDir, cargoTargetDir });
    await canonicalDirectory(tauriDir);
    await run({ command: "cargo", args: plan.cargoArgs, cwd: tauriDir });
    await copyVerifiedAtomic(plan.source, plan.destination, MAX_HELPER_BYTES);
  } catch {
    fail();
  }
}
