import { setTimeout as wait } from "node:timers/promises";
import { extractVerifiedArchive } from "./cef-extract.mjs";

const WINDOWS_EXTRACTION_ATTEMPTS = 2;
const WINDOWS_EXTRACTION_RETRY_DELAY_MS = 500;

export async function extractCefWithRetry(archive, artifact, options = {}) {
  const extract = options.extract ?? extractVerifiedArchive;
  const platform = options.platform ?? process.platform;
  const waitForRetry = options.wait ?? wait;
  if (typeof extract !== "function" || typeof waitForRetry !== "function") {
    throw new Error("CEF extraction failed");
  }

  const attempts = platform === "win32" ? WINDOWS_EXTRACTION_ATTEMPTS : 1;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      return await extract(archive, artifact);
    } catch (error) {
      if (attempt === attempts) throw error;
      await waitForRetry(WINDOWS_EXTRACTION_RETRY_DELAY_MS);
    }
  }
  throw new Error("CEF extraction failed");
}
