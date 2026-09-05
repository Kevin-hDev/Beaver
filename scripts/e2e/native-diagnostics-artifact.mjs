import { randomBytes } from "node:crypto";
import { mkdir, rename, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { collectNativeDiagnostics } from "./native-diagnostics.mjs";

const REPORT_NAME = "native-diagnostics.txt";
const ARTIFACT_ERROR = "Native diagnostic artifact failed";

export async function persistNativeDiagnostics(logDirectory, outputDirectory) {
  if (!validPath(logDirectory) || !validPath(outputDirectory)) {
    throw new Error(ARTIFACT_ERROR);
  }
  const diagnostics = await collectNativeDiagnostics(logDirectory);
  const report = diagnostics.length > 0
    ? `${diagnostics.join("\n")}\n`
    : "no-safe-failure-category\n";
  await mkdir(outputDirectory, { recursive: true, mode: 0o700 });
  const temporary = join(outputDirectory, temporaryName());
  const destination = join(outputDirectory, REPORT_NAME);
  try {
    await writeFile(temporary, report, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
    await rename(temporary, destination);
  } catch {
    await rm(temporary, { force: true }).catch(() => {});
    throw new Error(ARTIFACT_ERROR);
  }
}

function temporaryName() {
  const nonce = randomBytes(16);
  try {
    return `.native-diagnostics-${nonce.toString("hex")}.tmp`;
  } finally {
    nonce.fill(0);
  }
}

function validPath(value) {
  return typeof value === "string"
    && value.length > 0
    && value.length <= 32_768
    && !/[\0\r\n]/u.test(value);
}
