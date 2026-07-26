import { createHash, timingSafeEqual } from "node:crypto";
import { createReadStream } from "node:fs";
import { mkdir, open, rename, rm, stat } from "node:fs/promises";
import https from "node:https";
import { resolve } from "node:path";
import { tauriDir } from "./cef-artifacts.mjs";

const MAX_REDIRECTS = 3;
const REQUEST_TIMEOUT_MS = 30_000;

async function writeAll(handle, chunk) {
  let offset = 0;
  while (offset < chunk.length) {
    const result = await handle.write(chunk, offset, chunk.length - offset);
    if (result.bytesWritten < 1) throw new Error("CEF download failed");
    offset += result.bytesWritten;
  }
}

function digestMatches(actual, expected) {
  const left = Buffer.from(actual, "hex");
  const right = Buffer.from(expected, "hex");
  return left.length === right.length && timingSafeEqual(left, right);
}

async function verify(path, artifact) {
  const metadata = await stat(path);
  if (!metadata.isFile() || metadata.size !== artifact.bytes) return false;
  const hash = createHash("sha256");
  let bytes = 0;
  for await (const chunk of createReadStream(path)) {
    bytes += chunk.length;
    if (bytes > artifact.bytes) return false;
    hash.update(chunk);
  }
  return (
    bytes === artifact.bytes &&
    digestMatches(hash.digest("hex"), artifact.sha256)
  );
}

function request(url, allowedHosts, redirects = 0) {
  return new Promise((resolveRequest, reject) => {
    const parsed = new URL(url);
    if (
      parsed.protocol !== "https:" ||
      !allowedHosts.includes(parsed.hostname)
    ) {
      reject(new Error("CEF download was refused"));
      return;
    }
    const req = https.get(
      parsed,
      { headers: { "User-Agent": "Beaver CEF verifier" } },
      (res) => {
        if (
          res.statusCode >= 300 &&
          res.statusCode < 400 &&
          res.headers.location
        ) {
          res.resume();
          if (redirects >= MAX_REDIRECTS)
            reject(new Error("CEF download failed"));
          else
            request(
              new URL(res.headers.location, parsed).href,
              allowedHosts,
              redirects + 1,
            ).then(resolveRequest, reject);
          return;
        }
        if (res.statusCode !== 200) {
          res.resume();
          reject(new Error("CEF download failed"));
          return;
        }
        resolveRequest(res);
      },
    );
    req.setTimeout(REQUEST_TIMEOUT_MS, () =>
      req.destroy(new Error("CEF download failed")),
    );
    req.once("error", reject);
  });
}

async function download(path, artifact) {
  const temporary = `${path}.partial-${process.pid}`;
  await rm(temporary, { force: true });
  const handle = await open(temporary, "wx", 0o600);
  try {
    const response = await request(artifact.url, artifact.allowedHosts);
    const hash = createHash("sha256");
    let bytes = 0;
    for await (const chunk of response) {
      bytes += chunk.length;
      if (bytes > artifact.bytes) throw new Error("CEF download failed");
      hash.update(chunk);
      await writeAll(handle, chunk);
    }
    await handle.sync();
    if (
      bytes !== artifact.bytes ||
      !digestMatches(hash.digest("hex"), artifact.sha256)
    ) {
      throw new Error("CEF integrity verification failed");
    }
  } catch (error) {
    await handle.close().catch(() => {});
    await rm(temporary, { force: true });
    throw error;
  }
  await handle.close();
  await rename(temporary, path);
}

export async function verifiedDownload(artifact, cacheName) {
  if (![".cef-cache", ".cef-tool-cache"].includes(cacheName)) {
    throw new Error("CEF cache configuration is invalid");
  }
  const cacheDir = resolve(tauriDir, cacheName);
  const path = resolve(cacheDir, artifact.file);
  await mkdir(cacheDir, { recursive: true, mode: 0o700 });
  if (await verify(path, artifact).catch(() => false)) return path;
  await rm(path, { force: true });
  await download(path, artifact);
  if (!(await verify(path, artifact)))
    throw new Error("CEF integrity verification failed");
  return path;
}

export function verifiedArchive(artifact) {
  return verifiedDownload(artifact, ".cef-cache");
}
