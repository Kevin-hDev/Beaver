import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";

import { resolveArtifact } from "./resolve-artifact-path.mjs";

test("résout le NSIS Windows sans shell", () => {
  const cargoTargetDir = resolve("target", "release-windows");
  assert.deepEqual(
    resolveArtifact({
      tag: "v1.1.1",
      target: "x86_64-pc-windows-msvc",
      bundleDir: "nsis",
      suffix: "_x64-setup.exe",
      cargoTargetDir,
    }),
    {
      asset: join(
        cargoTargetDir,
        "x86_64-pc-windows-msvc",
        "release",
        "bundle",
        "nsis",
        "Beaver_1.1.1_x64-setup.exe",
      ),
    },
  );
});

test("refuse de deviner le dossier Cargo d'un bundle Windows", () => {
  assert.throws(
    () =>
      resolveArtifact({
        tag: "v1.1.1",
        target: "x86_64-pc-windows-msvc",
        bundleDir: "nsis",
        suffix: "_x64-setup.exe",
      }),
    /Artifact path resolution failed/,
  );
});

test("résout aussi l'application macOS exacte", () => {
  assert.deepEqual(
    resolveArtifact({
      tag: "v2.0.3",
      target: "aarch64-apple-darwin",
      bundleDir: "dmg",
      suffix: "_aarch64.dmg",
    }),
    {
      asset: "src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Beaver_2.0.3_aarch64.dmg",
      app: "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/Beaver.app",
    },
  );
});

test("refuse les entrées non strictes, traversées ou démesurées", () => {
  const valid = {
    tag: "v1.1.1",
    target: "x86_64-pc-windows-msvc",
    bundleDir: "nsis",
    suffix: "_x64-setup.exe",
  };
  for (const override of [
    { tag: "1.1.1" },
    { tag: "v01.1.1" },
    { tag: "v1.1.1-beta" },
    { target: "../windows" },
    { target: "x..windows" },
    { target: "x".repeat(513) },
    { bundleDir: "../nsis" },
    { suffix: ".exe" },
  ]) {
    assert.throws(
      () => resolveArtifact({ ...valid, ...override }),
      /Artifact path resolution failed/,
    );
  }
});

test("écrit les sorties GitHub bornées sans argument CLI", async () => {
  const root = await mkdtemp(join(tmpdir(), "beaver-artifact-path-"));
  const cargoTargetDir = join(root, "target");
  const output = join(root, "github-output.txt");
  try {
    await writeFile(output, "");
    await mkdir(cargoTargetDir);
    const canonicalCargoTargetDir = await realpath(cargoTargetDir);
    const canonicalOutput = await realpath(output);
    const result = spawnSync(process.execPath, ["scripts/release/resolve-artifact-path.mjs"], {
      cwd: process.cwd(),
      env: {
        ...process.env,
        RELEASE_TAG: "v1.1.1",
        BUNDLE_TARGET: "x86_64-pc-windows-msvc",
        BUNDLE_DIR: "nsis",
        ASSET_SUFFIX: "_x64-setup.exe",
        CARGO_TARGET_DIR: canonicalCargoTargetDir,
        GITHUB_OUTPUT: canonicalOutput,
      },
      shell: false,
      encoding: "utf8",
    });
    assert.equal(result.status, 0);
    assert.equal(
      await readFile(canonicalOutput, "utf8"),
      `asset=${join(canonicalCargoTargetDir, "x86_64-pc-windows-msvc", "release", "bundle", "nsis", "Beaver_1.1.1_x64-setup.exe")}\n`,
    );

    const rejected = spawnSync(
      process.execPath,
      ["scripts/release/resolve-artifact-path.mjs", "unexpected"],
      { cwd: process.cwd(), env: process.env, shell: false, encoding: "utf8" },
    );
    assert.notEqual(rejected.status, 0);
    assert.equal(rejected.stderr, "Artifact path resolution failed\n");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("refuse un fichier de sortie absent ou déjà trop grand", async () => {
  const root = await mkdtemp(join(tmpdir(), "beaver-artifact-output-"));
  const cargoTargetDir = join(root, "target");
  const missing = join(root, "missing.txt");
  const oversized = join(root, "oversized.txt");
  const environment = (output) => ({
    ...process.env,
    RELEASE_TAG: "v1.1.1",
    BUNDLE_TARGET: "x86_64-pc-windows-msvc",
    BUNDLE_DIR: "nsis",
    ASSET_SUFFIX: "_x64-setup.exe",
    CARGO_TARGET_DIR: cargoTargetDir,
    GITHUB_OUTPUT: output,
  });
  try {
    await mkdir(cargoTargetDir);
    await writeFile(oversized, Buffer.alloc(1024 * 1024));
    for (const output of [missing, oversized]) {
      const result = spawnSync(process.execPath, ["scripts/release/resolve-artifact-path.mjs"], {
        cwd: process.cwd(),
        env: environment(output),
        shell: false,
        encoding: "utf8",
      });
      assert.notEqual(result.status, 0);
      assert.equal(result.stderr, "Artifact path resolution failed\n");
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
