import { delimiter, dirname, resolve } from "node:path";

import { normalizeCargoTargetDir } from "../build/cargo-target-dir.mjs";

const MAX_ARGUMENTS = 64;
const MAX_ARGUMENT_LENGTH = 512;
const MAX_PATH_LENGTH = 30_000;

export function resolveCargoTargetDir({
  configuredTargetDir,
  platform,
  repoRoot,
}) {
  try {
    if (configuredTargetDir !== undefined) {
      return normalizeCargoTargetDir(configuredTargetDir);
    }
    if (platform !== "win32") return undefined;
    const normalizedRoot = normalizeCargoTargetDir(repoRoot);
    return normalizeCargoTargetDir(resolve(normalizedRoot, "target"));
  } catch {
    throw new Error("Tauri launch configuration is invalid");
  }
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
