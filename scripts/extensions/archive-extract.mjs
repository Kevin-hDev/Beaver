import { spawn } from "node:child_process";
import { writeFile } from "node:fs/promises";
import { join } from "node:path";

const POWERSHELL_SCRIPT = `param(
  [Parameter(Mandatory = $true)][string]$Archive,
  [Parameter(Mandatory = $true)][string]$Destination
)
$ErrorActionPreference = "Stop"
Expand-Archive -LiteralPath $Archive -DestinationPath $Destination
`;

export async function extractArchive(archive, destination, temporaryDirectory) {
  if (process.platform === "win32") {
    const scriptPath = join(temporaryDirectory, "extract-runtime.ps1");
    await writeFile(scriptPath, POWERSHELL_SCRIPT, { mode: 0o600 });
    await run("powershell.exe", windowsExtractionArguments(
      scriptPath,
      archive,
      destination,
    ));
    return;
  }
  await run("tar", ["-xzf", archive, "-C", destination]);
}

export function windowsExtractionArguments(scriptPath, archive, destination) {
  return [
    "-NoProfile",
    "-NonInteractive",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    scriptPath,
    archive,
    destination,
  ];
}

function run(program, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(program, args, {
      shell: false,
      stdio: ["ignore", "ignore", "ignore"],
    });
    child.once("error", reject);
    child.once("close", (code) => {
      if (code === 0) resolve();
      else reject(new Error("Runtime preparation failed"));
    });
  });
}
