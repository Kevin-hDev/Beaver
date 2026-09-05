import assert from "node:assert/strict";
import { lstat, mkdtemp, rm, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

test("the E2E artifact directory stays inside its dedicated repository root", async () => {
  const artifact = await import("./e2e-artifact-directory.mjs").catch(() => ({}));
  assert.equal(typeof artifact.prepareE2eArtifactDirectory, "function");
  if (typeof artifact.prepareE2eArtifactDirectory !== "function") return;
  const repository = await mkdtemp(join(tmpdir(), "beaver-e2e-repository-"));
  try {
    assert.equal(
      await artifact.prepareE2eArtifactDirectory(repository, undefined),
      undefined,
    );
    const directory = await artifact.prepareE2eArtifactDirectory(
      repository,
      ".e2e-artifacts/windows-cef",
    );
    assert.equal(directory, join(repository, ".e2e-artifacts", "windows-cef"));
    assert.equal((await lstat(directory)).isDirectory(), true);
    await assert.rejects(
      artifact.prepareE2eArtifactDirectory(repository, "../outside"),
      /E2E artifact directory is invalid/u,
    );
  } finally {
    await rm(repository, { recursive: true, force: true });
  }
});

test("the E2E artifact directory rejects a symbolic-link root", async (context) => {
  if (process.platform === "win32") {
    context.skip("Creating this test junction needs a Windows-specific target shape");
    return;
  }
  const repository = await mkdtemp(join(tmpdir(), "beaver-e2e-repository-"));
  const outside = await mkdtemp(join(tmpdir(), "beaver-e2e-outside-"));
  try {
    await symlink(outside, join(repository, ".e2e-artifacts"));
    const artifact = await import("./e2e-artifact-directory.mjs").catch(() => ({}));
    assert.equal(typeof artifact.prepareE2eArtifactDirectory, "function");
    if (typeof artifact.prepareE2eArtifactDirectory !== "function") return;
    await assert.rejects(
      artifact.prepareE2eArtifactDirectory(repository, ".e2e-artifacts/windows-cef"),
      /E2E artifact directory is invalid/u,
    );
  } finally {
    await rm(repository, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});
