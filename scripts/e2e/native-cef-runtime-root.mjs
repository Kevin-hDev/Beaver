import { posix, win32 } from "node:path";

const MAX_BINARY_PATH_CHARS = 1_024;
const FAILURE_MESSAGE = "Native CEF observation failed";

export function runtimeRootForBinary(platform, binaryPath) {
  invalid(typeof binaryPath !== "string"
    || binaryPath.length === 0
    || binaryPath.length > MAX_BINARY_PATH_CHARS);
  if (platform === "win32") {
    invalid(!win32.isAbsolute(binaryPath));
    return win32.dirname(binaryPath);
  }
  if (platform === "darwin") {
    const marker = ".app/Contents/MacOS/";
    const markerIndex = binaryPath.lastIndexOf(marker);
    invalid(!posix.isAbsolute(binaryPath) || markerIndex < 1);
    return binaryPath.slice(0, markerIndex + ".app".length);
  }
  throw new Error(FAILURE_MESSAGE);
}

function invalid(condition) {
  if (condition) throw new Error(FAILURE_MESSAGE);
}
