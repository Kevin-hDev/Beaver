import { spawn } from "node:child_process";
import { realpath } from "node:fs/promises";
import { isAbsolute, relative, resolve } from "node:path";
import {
  repoRoot,
  selectedBuildTool,
  tauriDir,
} from "./cef-artifacts.mjs";
import { prepareBuildTool } from "./cef-tool.mjs";
import { createTauriLaunch } from "./tauri-launch.mjs";

try {
  const cliPath = await trustedFile(
    resolve(repoRoot, "node_modules/@tauri-apps/cli/tauri.js"),
    repoRoot,
  );
  const tool = selectedBuildTool();
  const toolPath = tool
    ? await trustedFile(await prepareBuildTool(tool), tauriDir)
    : null;
  const launch = createTauriLaunch({
    args: process.argv.slice(2),
    cliPath,
    currentPath: process.env.PATH ?? "",
    executablePath: process.execPath,
    toolPath,
  });

  process.env.PATH = launch.path;
  process.exitCode = await run(launch);
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

function run(launch) {
  return new Promise((resolveRun, rejectRun) => {
    const child = spawn(launch.command, launch.args, {
      cwd: repoRoot,
      env: process.env,
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
