import { spawn } from "node:child_process";

const MAX_STDERR_CHARS = 2_048;
const MAX_ERROR_DETAIL_CHARS = 512;

export async function extractArchive(archive, destination) {
  if (process.platform === "win32") {
    await run(
      "tar.exe",
      windowsExtractionArguments(archive, destination),
      [archive, destination],
    );
    return;
  }
  await run("tar", ["-xzf", archive, "-C", destination], [archive, destination]);
}

export function windowsExtractionArguments(archive, destination) {
  return ["-xf", archive, "-C", destination];
}

export function sanitizeExtractionError(message, redactions) {
  let sanitized = String(message)
    .replace(/[\u0000-\u001f\u007f]+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  for (const value of redactions) {
    if (value) sanitized = sanitized.split(value).join("<path>");
  }
  return sanitized.slice(0, MAX_ERROR_DETAIL_CHARS);
}

function run(program, args, redactions) {
  return new Promise((resolve, reject) => {
    let stderr = "";
    const child = spawn(program, args, {
      shell: false,
      stdio: ["ignore", "ignore", "pipe"],
    });
    child.stderr.setEncoding("utf8");
    child.stderr.on("data", (chunk) => {
      const remaining = MAX_STDERR_CHARS - stderr.length;
      if (remaining > 0) stderr += String(chunk).slice(0, remaining);
    });
    child.once("error", () => reject(new Error("Runtime preparation failed")));
    child.once("close", (code) => {
      if (code === 0) resolve();
      else {
        const detail = sanitizeExtractionError(stderr, redactions);
        const suffix = detail ? `: ${detail}` : "";
        reject(new Error(`Runtime preparation failed${suffix}`));
      }
    });
  });
}
