import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  prepareNodeRuntime,
  readBoundedResponse,
  verifyChecksum,
} from "./node-runtime.mjs";

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

test("host preparation accepts only its explicit development flag", () => {
  const script = resolve(root, "scripts/extensions/prepare-extension-host.mjs");
  const result = spawnSync(process.execPath, [script, "--unexpected"], {
    encoding: "utf8",
    shell: false,
  });

  assert.notEqual(result.status, 0);
});

test("the bundled host has no local dependency symlink", async () => {
  const packagePath = resolve(
    root,
    "src-tauri/resources/extension-host/package.json",
  );
  const packageData = JSON.parse(await readFile(packagePath, "utf8"));

  assert.deepEqual(packageData.dependencies, { jiti: "2.7.0" });
});
