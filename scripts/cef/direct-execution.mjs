import { realpathSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

function canonicalPath(path) {
  let canonical;
  try {
    canonical = realpathSync.native(resolve(path));
  } catch {
    canonical = resolve(path);
  }
  return process.platform === "win32" ? canonical.toLocaleLowerCase("en-US") : canonical;
}

export function isDirectExecution(moduleUrl, argvPath) {
  if (!argvPath) return false;
  return canonicalPath(fileURLToPath(moduleUrl)) === canonicalPath(argvPath);
}
