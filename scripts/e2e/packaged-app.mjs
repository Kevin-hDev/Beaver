import { readdir, lstat } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { debugBinaryPath } from "./e2e-process.mjs";

const PREPARATION_ERROR = "Packaged E2E preparation failed";
const MAX_INSTALLERS = 16;

export function windowsInstallerDirectory(cargoTargetDir) {
  return resolve(cargoTargetDir, "debug", "bundle", "nsis");
}

export async function preparePackagedApp({
  platform,
  cargoTargetDir,
  profilePath,
  listFiles = listWindowsInstallers,
  isRegularFile = regularFile,
  run,
}) {
  if (platform === "darwin") {
    const binaryPath = debugBinaryPath(platform, cargoTargetDir);
    if (!(await isRegularFile(binaryPath))) throw new Error(PREPARATION_ERROR);
    return { binaryPath, cleanup: async () => {} };
  }
  if (platform !== "win32" || typeof run !== "function") {
    throw new Error(PREPARATION_ERROR);
  }

  const installerDirectory = windowsInstallerDirectory(cargoTargetDir);
  const installers = await listFiles(installerDirectory);
  if (installers.length !== 1 || !(await isRegularFile(installers[0]))) {
    throw new Error(PREPARATION_ERROR);
  }
  const installDir = resolve(profilePath, "packaged-app");
  if (dirname(installDir) !== resolve(profilePath)) throw new Error(PREPARATION_ERROR);
  const installExit = await run(installers[0], ["/S", `/D=${installDir}`]);
  const binaryPath = join(installDir, "cl-go-dash.exe");
  if (installExit !== 0 || !(await isRegularFile(binaryPath))) {
    throw new Error(PREPARATION_ERROR);
  }

  return {
    binaryPath,
    cleanup: async () => {
      const uninstallExit = await run(join(installDir, "uninstall.exe"), ["/S"]);
      if (uninstallExit !== 0) throw new Error(PREPARATION_ERROR);
    },
  };
}

async function listWindowsInstallers(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  if (entries.length > MAX_INSTALLERS) throw new Error(PREPARATION_ERROR);
  return entries
    .filter((entry) => entry.isFile() && /^Beaver_[0-9]+\.[0-9]+\.[0-9]+_x64-setup\.exe$/u.test(entry.name))
    .map((entry) => join(directory, entry.name));
}

async function regularFile(path) {
  try {
    const metadata = await lstat(path);
    return metadata.isFile() && !metadata.isSymbolicLink();
  } catch {
    return false;
  }
}
