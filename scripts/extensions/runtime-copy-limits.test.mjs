import assert from "node:assert/strict";
import {
  mkdir,
  mkdtemp,
  rm,
  truncate,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  COMPLETE_RUNTIME_COPY_LIMITS,
  DEPENDENCY_COPY_LIMITS,
} from "./runtime-copy-limits.mjs";
import { copyDirectoryBounded } from "./runtime-copy.mjs";
import {
  ensureCachedRuntime,
  materializeRuntime,
  runtimeIsValid,
} from "./runtime-cache.mjs";

test("complete runtimes can exceed the dependency-only byte limit", async () => {
  const temporary = await mkdtemp(join(tmpdir(), "beaver-large-runtime-"));
  const cache = join(temporary, "cache");
  const destination = join(temporary, "host", "runtime");
  const descriptor = {
    version: "1.0.0",
    platform: process.platform,
    architecture: process.arch,
    checksum: "b".repeat(64),
  };
  try {
    const cached = await ensureCachedRuntime(
      cache,
      "large-node-fixture",
      descriptor,
      async (directory) => {
        const executable = join(
          directory,
          process.platform === "win32" ? "node.exe" : "node",
        );
        await mkdir(join(directory, "npm/bin"), { recursive: true });
        await writeFile(executable, "node");
        await truncate(executable, DEPENDENCY_COPY_LIMITS.maxBytes + 1);
        await writeFile(join(directory, "npm/bin/npm-cli.js"), "npm");
        await writeFile(join(directory, "NODE_LICENSE"), "license");
      },
    );

    await materializeRuntime(cached, destination, descriptor);

    assert.equal(await runtimeIsValid(destination, descriptor), true);
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
});

test("complete runtimes remain bounded", async () => {
  const temporary = await mkdtemp(join(tmpdir(), "beaver-oversized-runtime-"));
  const source = join(temporary, "source");
  const destination = join(temporary, "destination");
  try {
    await mkdir(source);
    const oversized = join(source, "node");
    await writeFile(oversized, "node");
    await truncate(oversized, COMPLETE_RUNTIME_COPY_LIMITS.maxBytes + 1);

    await assert.rejects(
      copyDirectoryBounded(source, destination, COMPLETE_RUNTIME_COPY_LIMITS),
      /too large/,
    );
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
});
