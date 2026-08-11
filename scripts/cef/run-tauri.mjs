import { spawn } from "node:child_process";
import { realpath } from "node:fs/promises";
import { isAbsolute, relative, resolve } from "node:path";
import {
  repoRoot,
  selectedBuildTool,
  tauriDir,
} from "./cef-artifacts.mjs";
import { prepareBuildTool } from "./cef-tool.mjs";
import {
  createTauriLaunch,
  resolveCargoTargetDir,
} from "./tauri-launch.mjs";
import { resolveWindowsBundleRequest } from "./tauri-bundle-request.mjs";

try {
  const cliPath = await trustedFile(
    resolve(repoRoot, "node_modules/@tauri-apps/cli/tauri.js"),
    repoRoot,
  );
  const tool = selectedBuildTool();
  const toolPath = tool
    ? await trustedFile(await prepareBuildTool(tool), tauriDir)
    : null;
  const bundleRequest = resolveWindowsBundleRequest({
    args: process.argv.slice(2),
    platform: process.platform,
  });
  const launch = createTauriLaunch({
    args: bundleRequest.args,
    cliPath,
    currentPath: process.env.PATH ?? "",
    executablePath: process.execPath,
    toolPath,
  });
  const environment = { ...process.env, PATH: launch.path };
  if (typeof process.env.npm_execpath === "string") {
    environment.BEAVER_NPM_CLI_PATH = process.env.npm_execpath;
  }
  const cargoTargetDir = resolveCargoTargetDir({
    configuredTargetDir: process.env.CARGO_TARGET_DIR,
    platform: process.platform,
    repoRoot,
  });
  if (cargoTargetDir !== undefined) {
    environment.CARGO_TARGET_DIR = cargoTargetDir;
  }
  if (bundleRequest.bundleType === null) {
    delete environment.BEAVER_TAURI_BUNDLE_TYPE;
  } else {
    environment.BEAVER_TAURI_BUNDLE_TYPE = bundleRequest.bundleType;
  }

  process.exitCode = await run(launch, environment);
} catch {
  console.error("Tauri preparation failed");
  process.exitCode = 1;
}

async function trustedFile(candidate, parent) {
  const [filePath, parentPath] = await Promise.all([
    realpath(candidate),
    realpath(parent),
  ]);
  const childPath = relative(parentPath, filePath);
  if (!childPath || childPath.startsWith("..") || isAbsolute(childPath)) {
    throw new Error("Tauri preparation failed");
  }
  return filePath;
}

function run(launch, environment) {
  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(launch.command, launch.args, {
      cwd: repoRoot,
      env: environment,
      shell: false,
      stdio: "inherit",
      windowsHide: true,
    });
    child.once("error", rejectRun);
    child.once("exit", (code, signal) => {
      resolveRun(signal === null && Number.isInteger(code) ? code : 1);
    });
  });
}
