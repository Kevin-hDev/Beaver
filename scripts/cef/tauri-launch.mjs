import { delimiter, dirname } from "node:path";

const MAX_ARGUMENTS = 64;
const MAX_ARGUMENT_LENGTH = 512;
const MAX_PATH_LENGTH = 30_000;

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
