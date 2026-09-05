import { constants } from "node:fs";
import { open, realpath } from "node:fs/promises";
import { basename, join } from "node:path";
import { tmpdir } from "node:os";

// Disposable investigation: remove once the packaged Windows activation is explained.
export const DIAGNOSTIC_POLL_MS = 500;
export const MAX_DIAGNOSTIC_SAMPLES = 1_024;
const MAX_MARKER_BYTES = 2_048;
const STAGES = new Set(["import", "activate", "register"]);
const IDENTIFIER = /^[a-zA-Z0-9][a-zA-Z0-9._-]{0,126}[a-zA-Z0-9]$/u;

export async function diagnosticProfile(value) {
  if (typeof value !== "string" || value.length > 32_768 || value.includes("..")) {
    throw new Error("Diagnostic profile unavailable");
  }
  const profile = await realpath(value);
  const temporary = await realpath(tmpdir());
  if (!profile.startsWith(`${temporary}/`) && !profile.startsWith(`${temporary}\\`)) {
    throw new Error("Diagnostic profile unavailable");
  }
  if (!/^beaver-e2e-[a-zA-Z0-9]+$/u.test(basename(profile))) {
    throw new Error("Diagnostic profile unavailable");
  }
  return profile;
}

export function markerSummary(bytes) {
  if (bytes.length > MAX_MARKER_BYTES) return { state: "oversized" };
  try {
    const value = JSON.parse(bytes.toString("utf8"));
    const marker = value.version === 2 ? value.host : value;
    if (!marker) return { state: "no-host-marker" };
    if (!IDENTIFIER.test(marker.extensionId) || !STAGES.has(marker.stage)) {
      return { state: "invalid" };
    }
    return { state: "present", extensionId: marker.extensionId, stage: marker.stage };
  } catch {
    return { state: "invalid" };
  }
}

export async function sampleMarker(profile) {
  let file;
  try {
    file = await open(join(profile, "extension-loading.json"), constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0));
    const metadata = await file.stat();
    if (!metadata.isFile() || metadata.nlink !== 1 || metadata.size > MAX_MARKER_BYTES) {
      return { state: "invalid-file" };
    }
    const buffer = Buffer.alloc(MAX_MARKER_BYTES + 1);
    const { bytesRead } = await file.read(buffer);
    return markerSummary(buffer.subarray(0, bytesRead));
  } catch (error) {
    return { state: error?.code === "ENOENT" ? "missing" : "read-failed" };
  } finally {
    await file?.close();
  }
}

export function safeOutcome(error) {
  return typeof error?.message === "string" && /^extensions_[a-z_]{1,64}$/u.test(error.message)
    ? error.message : "operation-failed";
}

export async function verifiedScriptTimeout(driver, requested) {
  const previous = await driver.getTimeouts();
  await driver.setTimeout({ script: requested });
  const actual = await driver.getTimeouts();
  if (actual.script !== requested) throw new Error("Diagnostic timeout not applied");
  return { previous: previous.script, effective: actual.script };
}
