import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  prepareNodeRuntime,
  readBoundedResponse,
  validateArtifactTable,
  verifyChecksum,
} from "./node-runtime.mjs";
import {
  sanitizeExtractionError,
  windowsExtractionArguments,
} from "./archive-extract.mjs";
import { copyDirectoryBounded } from "./runtime-copy.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

test("runtime preparation rejects unsafe targets before downloading", async () => {
  await assert.rejects(
    prepareNodeRuntime("relative/extension-host"),
    /Invalid extension host directory/,
  );
});

test("runtime downloads and checksums fail closed", async () => {
  const oversized = new Response(null, {
    headers: { "content-length": String(100 * 1024 * 1024 + 1) },
  });
  await assert.rejects(readBoundedResponse(oversized), /too large/);

  const bytes = Buffer.from("beaver-runtime");
  const checksum = createHash("sha256").update(bytes).digest("hex");
  assert.doesNotThrow(() => verifyChecksum(bytes, checksum));
  assert.throws(() => verifyChecksum(bytes, "0".repeat(64)), /Invalid Node.js checksum/);
});

test("all bundled Node.js checksums are valid SHA-256 values", () => {
  assert.doesNotThrow(() => validateArtifactTable());
});

test("bundled npm copying is explicitly bounded", async () => {
  const temporary = await mkdtemp(join(tmpdir(), "beaver-runtime-copy-"));
  const source = join(temporary, "source");
  const destination = join(temporary, "destination");
  try {
    await mkdir(source);
    await writeFile(join(source, "one.js"), "one");
    await writeFile(join(source, "two.js"), "two");
    await assert.rejects(
      copyDirectoryBounded(source, destination, {
        maxEntries: 1,
        maxBytes: 1024,
        maxDepth: 4,
      }),
      /too many entries/,
    );
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
});

test("Windows extraction uses tar.exe arguments without a shell or script", () => {
  const arguments_ = windowsExtractionArguments(
    "C:\\temp\\node.zip",
    "C:\\temp\\runtime",
  );

  assert.deepEqual(arguments_, [
    "-xf",
    "C:\\temp\\node.zip",
    "-C",
    "C:\\temp\\runtime",
  ]);
});

test("archive extraction diagnostics are bounded and redact input paths", () => {
  const archive = "C:\\temp\\node.zip";
  const detail = sanitizeExtractionError(
    `tar.exe:\n${archive}\twas not found`,
    [archive],
  );

  assert.equal(detail, "tar.exe: <path> was not found");
  assert.equal(detail.includes(archive), false);
  assert.ok(detail.length <= 512);
});

test("host preparation accepts only its explicit development flag", () => {
  const script = resolve(root, "scripts/extensions/prepare-extension-host.mjs");
  const result = spawnSync(process.execPath, [script, "--unexpected"], {
    encoding: "utf8",
    shell: false,
  });

  assert.notEqual(result.status, 0);
});

test("the bundled host uses the same exact jiti version as Beaver", async () => {
  const hostPackagePath = resolve(
    root,
    "src-tauri/resources/extension-host/package.json",
  );
  const rootPackagePath = resolve(root, "package.json");
  const [hostPackage, rootPackage] = await Promise.all([
    readFile(hostPackagePath, "utf8").then(JSON.parse),
    readFile(rootPackagePath, "utf8").then(JSON.parse),
  ]);

  assert.equal(
    hostPackage.dependencies.jiti,
    rootPackage.devDependencies.jiti,
  );
  assert.match(hostPackage.dependencies.jiti, /^\d+\.\d+\.\d+$/);
});
