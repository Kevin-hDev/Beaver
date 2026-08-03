import { realpath, stat } from "node:fs/promises";
import { basename, dirname, isAbsolute, relative } from "node:path";

const MAX_NPM_ARGUMENTS = 16;
const MAX_NPM_ARGUMENT_LENGTH = 4096;
const NPM_CLI_FILENAME = "npm-cli.js";

export async function createNpmInvocation(
  args,
  npmCliPath = process.env.npm_execpath,
) {
  if (!validArguments(args) || !validCliPath(npmCliPath)) {
    throw new Error("Invalid npm runtime");
  }

  try {
    const canonicalParent = await realpath(dirname(npmCliPath));
    const canonicalCli = await realpath(npmCliPath);
    const metadata = await stat(canonicalCli);
    const parentRelativePath = relative(canonicalParent, canonicalCli);

    if (
      !metadata.isFile()
      || basename(canonicalCli) !== NPM_CLI_FILENAME
      || parentRelativePath !== NPM_CLI_FILENAME
    ) {
      throw new Error("Invalid npm runtime");
    }

    return {
      program: process.execPath,
      args: [canonicalCli, ...args],
    };
  } catch {
    throw new Error("Invalid npm runtime");
  }
}

function validArguments(args) {
  return Array.isArray(args)
    && args.length <= MAX_NPM_ARGUMENTS
    && args.every(
      (argument) =>
        typeof argument === "string"
        && argument.length > 0
        && argument.length <= MAX_NPM_ARGUMENT_LENGTH
        && !argument.includes("\0"),
    );
}

function validCliPath(npmCliPath) {
  return typeof npmCliPath === "string"
    && npmCliPath.length > 0
    && npmCliPath.length <= MAX_NPM_ARGUMENT_LENGTH
    && !npmCliPath.includes("\0")
    && !npmCliPath.split(/[\\/]/u).includes("..")
    && isAbsolute(npmCliPath);
}
