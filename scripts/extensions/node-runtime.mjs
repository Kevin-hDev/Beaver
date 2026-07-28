import { createHash } from "node:crypto";
import {
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { extractArchive } from "./archive-extract.mjs";
import { copyDirectoryBounded } from "./runtime-copy.mjs";

const VERSION = "24.18.0";
const MAX_RUNTIME_ARCHIVE_BYTES = 100 * 1024 * 1024;
const PROJECT_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const ALLOWED_HOST_DIRECTORIES = new Set([
  resolve(PROJECT_ROOT, "src-tauri/resources/extension-host"),
  resolve(PROJECT_ROOT, "src-tauri/target/extension-host"),
]);
const ARTIFACTS = {
  "darwin-arm64": {
    archive: `node-v${VERSION}-darwin-arm64.tar.gz`,
    checksum: "e1a97e14c99c803e96c7339403282ea05a499c32f8d83defe9ef5ec66f979ed1",
    executable: `node-v${VERSION}-darwin-arm64/bin/node`,
  },
  "darwin-x64": {
    archive: `node-v${VERSION}-darwin-x64.tar.gz`,
    checksum: "dfd0dbd3e721503434df7b7205e719f61b3a3a31b2bcf9729b8b91fea240f080",
    executable: `node-v${VERSION}-darwin-x64/bin/node`,
  },
  "linux-arm64": {
    archive: `node-v${VERSION}-linux-arm64.tar.gz`,
    checksum: "6b4484c2190274175df9aa8f28e2d758a819cb1c1fe6ab481e2f95b463ab8508",
    executable: `node-v${VERSION}-linux-arm64/bin/node`,
  },
  "linux-x64": {
    archive: `node-v${VERSION}-linux-x64.tar.gz`,
    checksum: "783130984963db7ba9cbd01089eaf2c2efb055c7c1693c943174b967b3050cb8",
    executable: `node-v${VERSION}-linux-x64/bin/node`,
  },
  "win32-arm64": {
    archive: `node-v${VERSION}-win-arm64.zip`,
    checksum: "f274669adb93b1fd0fbf8f21fd078609e9dcc84333d4f2718d2dde3f9a161a01",
    executable: `node-v${VERSION}-win-arm64/node.exe`,
  },
  "win32-x64": {
    archive: `node-v${VERSION}-win-x64.zip`,
    checksum: "0ae68406b42d7725661da979b1403ec9926da205c6770827f33aac9d8f26e821",
    executable: `node-v${VERSION}-win-x64/node.exe`,
  },
};

export async function prepareNodeRuntime(hostDirectory) {
  if (typeof hostDirectory !== "string" || !ALLOWED_HOST_DIRECTORIES.has(hostDirectory)) {
    throw new Error("Invalid extension host directory");
  }
  const artifact = ARTIFACTS[`${process.platform}-${process.arch}`];
  if (!artifact) throw new Error("Unsupported extension host platform");
  validateArtifactTable();
  const runtime = join(hostDirectory, "runtime");
  const destination = join(runtime, process.platform === "win32" ? "node.exe" : "node");

  const temporary = await mkdtemp(join(tmpdir(), "beaver-node-runtime-"));
  try {
    const archivePath = join(temporary, artifact.archive);
    const response = await fetch(`https://nodejs.org/dist/v${VERSION}/${artifact.archive}`, {
      redirect: "error",
      signal: AbortSignal.timeout(120_000),
    });
    if (!response.ok) throw new Error("Unable to download the Node.js runtime");
    const bytes = await readBoundedResponse(response);
    verifyChecksum(bytes, artifact.checksum);
    await writeFile(archivePath, bytes, { mode: 0o600 });
    const extracted = join(temporary, "extracted");
    await mkdir(extracted);
    await extractArchive(archivePath, extracted, temporary);
    await rm(runtime, { recursive: true, force: true });
    await mkdir(runtime, { recursive: true, mode: 0o700 });
    await copyFile(join(extracted, artifact.executable), destination);
    if (process.platform !== "win32") await chmod(destination, 0o700);
    const archiveRoot = artifact.executable.split("/")[0];
    const npmSource = process.platform === "win32"
      ? join(extracted, archiveRoot, "node_modules", "npm")
      : join(extracted, archiveRoot, "lib", "node_modules", "npm");
    await copyDirectoryBounded(npmSource, join(runtime, "npm"));
    await copyFile(join(extracted, archiveRoot, "LICENSE"), join(runtime, "NODE_LICENSE"));
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
}

export async function readBoundedResponse(response) {
  const declared = Number(response.headers.get("content-length"));
  if (Number.isFinite(declared) && declared > MAX_RUNTIME_ARCHIVE_BYTES) {
    throw new Error("Node.js runtime archive is too large");
  }
  if (!response.body) throw new Error("Node.js runtime archive is unavailable");
  const reader = response.body.getReader();
  const chunks = [];
  let size = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    size += value.byteLength;
    if (size > MAX_RUNTIME_ARCHIVE_BYTES) {
      await reader.cancel();
      throw new Error("Node.js runtime archive is too large");
    }
    chunks.push(Buffer.from(value));
  }
  return Buffer.concat(chunks, size);
}

export function verifyChecksum(bytes, expected) {
  const actual = createHash("sha256").update(bytes).digest("hex");
  if (actual.length !== expected.length) throw new Error("Invalid Node.js checksum");
  let difference = 0;
  for (let index = 0; index < actual.length; index += 1) {
    difference |= actual.charCodeAt(index) ^ expected.charCodeAt(index);
  }
  if (difference !== 0) throw new Error("Invalid Node.js checksum");
}

export function validateArtifactTable() {
  for (const artifact of Object.values(ARTIFACTS)) {
    if (!/^[0-9a-f]{64}$/.test(artifact.checksum)) {
      throw new Error("Invalid Node.js artifact checksum");
    }
  }
}
