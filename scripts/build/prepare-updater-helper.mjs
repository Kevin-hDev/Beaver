import { lstat, rm } from "node:fs/promises";
import { join } from "node:path";

import { runCommand } from "./command-runner.mjs";
import {
  canonicalDirectory,
  copyVerifiedAtomic,
  MAX_HELPER_BYTES,
  validateAbsolutePath,
} from "./updater-helper-copy.mjs";

const ERROR_MESSAGE = "Updater helper preparation failed";
const TARGET_PATTERN = /^[A-Za-z0-9_.-]{1,128}$/u;
const PLATFORMS = new Set(["darwin", "linux", "win32"]);

function fail() {
  throw new Error(ERROR_MESSAGE);
}

export { copyVerifiedAtomic };

export function createUpdaterBuildPlan({ platform, target = "", tauriDir } = {}) {
  try {
    if (
      !PLATFORMS.has(platform) ||
      typeof target !== "string" ||
      target === "." ||
      target === ".." ||
      (target && !TARGET_PATTERN.test(target))
    ) {
      fail();
    }
    validateAbsolutePath(tauriDir);
    const cargoArgs = ["build", "--release", "--bin", "cl-go-dash-updater"];
    if (target) cargoArgs.push("--target", target);
    const windows = target ? target.includes("-windows-") : platform === "win32";
    const filename = `cl-go-dash-updater${windows ? ".exe" : ""}`;
    const targetRoot = target ? join(tauriDir, "target", target) : join(tauriDir, "target");
    return {
      cargoArgs,
      source: join(targetRoot, "release", filename),
      destination: join(tauriDir, "target", "updater-helper", filename),
      staleDestination: join(
        tauriDir,
        "target",
        "updater-helper",
        windows ? "cl-go-dash-updater" : "cl-go-dash-updater.exe",
      ),
    };
  } catch {
    fail();
  }
}

async function removeStaleHelper(path) {
  try {
    const stale = await lstat(path);
    if (!stale.isFile() || stale.isSymbolicLink() || stale.nlink > 1 || stale.size > MAX_HELPER_BYTES) fail();
    await rm(path);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
}

export async function prepareUpdaterHelper({ platform, target = "", tauriDir, run = runCommand } = {}) {
  try {
    if (typeof run !== "function") fail();
    const plan = createUpdaterBuildPlan({ platform, target, tauriDir });
    await canonicalDirectory(tauriDir);
    await run({ command: "cargo", args: plan.cargoArgs, cwd: tauriDir });
    await copyVerifiedAtomic(plan.source, plan.destination, MAX_HELPER_BYTES);
    await removeStaleHelper(plan.staleDestination);
  } catch {
    fail();
  }
}
