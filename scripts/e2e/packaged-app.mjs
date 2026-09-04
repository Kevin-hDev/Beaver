import { readdir, lstat } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";

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
  listLinuxFiles = listLinuxBundles,
  isRegularFile = regularFile,
  run,
}) {
  if (typeof run !== "function") {
    throw new Error(PREPARATION_ERROR);
  }
  if (platform === "darwin") {
    const binaryPath = resolve(
      cargoTargetDir,
      "debug", "bundle", "macos", "Beaver.app", "Contents", "MacOS", "cl-go-dash",
    );
    if (!(await isRegularFile(binaryPath))) throw new Error(PREPARATION_ERROR);
    return { binaryPath, cleanup: async () => {} };
  }
  if (platform === "linux") {
    const directory = resolve(cargoTargetDir, "debug", "bundle", "appimage");
    let bundles;
    try {
      bundles = await listLinuxFiles(directory);
    } catch {
      throw new Error(PREPARATION_ERROR);
    }
    if (bundles.length !== 1 || !(await isRegularFile(bundles[0]))) {
      throw new Error(PREPARATION_ERROR);
    }
    return { binaryPath: bundles[0], cleanup: async () => {} };
  }
  if (platform !== "win32") throw new Error(PREPARATION_ERROR);

  let installers;
  try {
    installers = await listFiles(windowsInstallerDirectory(cargoTargetDir));
  } catch {
    throw new Error(PREPARATION_ERROR);
  }
  if (installers.length !== 1 || !(await isRegularFile(installers[0]))) {
    throw new Error(PREPARATION_ERROR);
  }
  const installDir = resolve(profilePath, "packaged-app");
  if (dirname(installDir) !== resolve(profilePath)) throw new Error(PREPARATION_ERROR);
  const binaryPath = join(installDir, "cl-go-dash.exe");
  const runOptions = { windowsVerbatimArguments: true };
  try {
    const installExit = await run(
      installers[0],
      ["/S", `/D=${installDir}`],
      runOptions,
    );
    if (installExit !== 0 || !(await isRegularFile(binaryPath))) {
      throw new Error(PREPARATION_ERROR);
    }
  } catch {
    await tryRemoveWindowsInstallation({ installDir, isRegularFile, run, runOptions });
    throw new Error(PREPARATION_ERROR);
  }

  return {
    binaryPath,
    cleanup: async () => {
      if (!(await removeWindowsInstallation({ installDir, isRegularFile, run, runOptions }))) {
        throw new Error(PREPARATION_ERROR);
      }
    },
  };
}

async function listLinuxBundles(directory) {
  try {
    const entries = await readdir(directory, { withFileTypes: true });
    if (entries.length > MAX_INSTALLERS) throw new Error(PREPARATION_ERROR);
    return entries
      .filter((entry) => (
        entry.isFile()
        && /^Beaver_[0-9]+\.[0-9]+\.[0-9]+_amd64\.AppImage$/u.test(entry.name)
      ))
      .map((entry) => join(directory, entry.name));
  } catch {
    throw new Error(PREPARATION_ERROR);
  }
}

async function tryRemoveWindowsInstallation(context) {
  try {
    return await removeWindowsInstallation(context);
  } catch {
    return false;
  }
}

async function removeWindowsInstallation({ installDir, isRegularFile, run, runOptions }) {
  const uninstaller = join(installDir, "uninstall.exe");
  if (!(await isRegularFile(uninstaller))) return false;
  const exit = await run(
    uninstaller,
    ["/S", `_?=${installDir}`],
    runOptions,
  );
  return exit === 0;
}

async function listWindowsInstallers(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  if (entries.length > MAX_INSTALLERS) throw new Error(PREPARATION_ERROR);
  return entries
    .filter((entry) => (
      entry.isFile()
      && isWindowsInstallerName(entry.name)
    ))
    .map((entry) => join(directory, entry.name));
}

export function isWindowsInstallerName(name) {
  return typeof name === "string"
    && /^Beaver(?:_| )E2E_[0-9]+\.[0-9]+\.[0-9]+_x64-setup\.exe$/u.test(name);
}

async function regularFile(path) {
  try {
    const metadata = await lstat(path);
    return metadata.isFile() && !metadata.isSymbolicLink();
  } catch {
    return false;
  }
}
