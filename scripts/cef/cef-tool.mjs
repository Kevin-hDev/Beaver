import { spawnSync } from "node:child_process";
import { chmod, mkdir, rename, rm, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { tauriDir } from "./cef-artifacts.mjs";
import { verifiedDownload } from "./cef-download.mjs";

const MAX_TOOL_BYTES = 2 * 1024 * 1024;

export async function prepareBuildTool(tool) {
  const archive = await verifiedDownload(tool, ".cef-tool-cache");
  const directory = resolve(tauriDir, ".cef-tools");
  const destination = resolve(directory, tool.output);
  const temporary = `${destination}.partial-${process.pid}`;
  await mkdir(directory, { recursive: true, mode: 0o700 });
  await rm(temporary, { force: true });

  const result = spawnSync("tar", ["-xOf", archive, tool.entry], {
    encoding: null,
    maxBuffer: MAX_TOOL_BYTES,
    shell: false,
    timeout: 30_000,
    windowsHide: true,
  });
  if (result.status !== 0 || result.error || result.stdout.length < 1) {
    throw new Error("CEF build tool preparation failed");
  }
  await writeFile(temporary, result.stdout, { flag: "wx", mode: 0o700 });
  await chmod(temporary, 0o700);
  await rm(destination, { force: true });
  await rename(temporary, destination);
  if (process.platform === "darwin") {
    const wrapper = resolve(directory, "cmake-wrapper");
    const script = [
      "#!/bin/sh",
      "set -eu",
      'TOOLS_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)"',
      'case "${1-}" in',
      '  --build|--install) exec cmake "$@" ;;',
      "esac",
      'exec cmake "$@" "-DCMAKE_MAKE_PROGRAM=$TOOLS_DIR/ninja"',
      "",
    ].join("\n");
    await writeFile(wrapper, script, { mode: 0o700 });
    await chmod(wrapper, 0o700);
  }
  return destination;
}
