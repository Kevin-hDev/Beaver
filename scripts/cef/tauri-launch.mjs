import { delimiter, dirname, resolve } from "node:path";

const MAX_ARGUMENTS = 64;
const MAX_ARGUMENT_LENGTH = 512;
const MAX_PATH_LENGTH = 30_000;

export function resolveCargoTargetDir({
  configuredTargetDir,
  platform,
  repoRoot,
}) {
  if (configuredTargetDir !== undefined) {
    if (
      typeof configuredTargetDir !== "string" ||
      configuredTargetDir.length > MAX_PATH_LENGTH ||
      /[\0\r\n]/u.test(configuredTargetDir)
    ) {
      throw new Error("Tauri launch configuration is invalid");
    }
    return configuredTargetDir;
  }

  if (platform !== "win32") return undefined;
  if (!validText(repoRoot, MAX_PATH_LENGTH)) {
    throw new Error("Tauri launch configuration is invalid");
  }

  const targetDir = resolve(repoRoot, "target");
  if (!validText(targetDir, MAX_PATH_LENGTH)) {
    throw new Error("Tauri launch configuration is invalid");
  }
  return targetDir;
}

export function createTauriLaunch({
  args,
  cliPath,
  currentPath,
  executablePath,
  toolPath,
}) {
  if (
    !Array.isArray(args) ||
    args.length > MAX_ARGUMENTS ||
    args.some(
      (argument) =>
        typeof argument !== "string" ||
        argument.length < 1 ||
        argument.length > MAX_ARGUMENT_LENGTH ||
        /[\0\r\n]/u.test(argument),
    ) ||
    !validText(cliPath) ||
    !validText(currentPath, MAX_PATH_LENGTH) ||
    !validText(executablePath) ||
    (toolPath !== null && !validText(toolPath))
  ) {
    throw new Error("Tauri launch configuration is invalid");
  }

  const path = toolPath
    ? `${dirname(toolPath)}${delimiter}${currentPath}`
    : currentPath;
  if (path.length > MAX_PATH_LENGTH) {
    throw new Error("Tauri launch configuration is invalid");
  }

  return Object.freeze({
    args: Object.freeze([cliPath, ...args]),
    command: executablePath,
    path,
  });
}

function validText(value, maxLength = 4_096) {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    value.length <= maxLength &&
    !/[\0\r\n]/u.test(value)
  );
}
