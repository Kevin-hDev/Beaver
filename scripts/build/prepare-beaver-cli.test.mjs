import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";

import { createBeaverCliBuildPlan, prepareBeaverCli } from "./prepare-beaver-cli.mjs";

test("prépare le nom ciblé attendu par Tauri", () => {
  const tauriDir = process.cwd();
  const plan = createBeaverCliBuildPlan({
    platform: "darwin",
    target: "aarch64-apple-darwin",
    tauriDir,
  });

  assert.deepEqual(plan.cargoArgs, [
    "build", "--release", "--bin", "beaver", "--target", "aarch64-apple-darwin",
  ]);
  assert.equal(plan.source, join(tauriDir, "target", "aarch64-apple-darwin", "release", "beaver"));
  assert.equal(plan.destination, join(tauriDir, "target", "beaver-cli", "beaver-aarch64-apple-darwin"));
});

test("compile puis copie le binaire vérifié", async () => {
  const tauriDir = await realpath(await mkdtemp(join(tmpdir(), "beaver-cli-")));
  try {
    const target = "x86_64-unknown-linux-gnu";
    const plan = createBeaverCliBuildPlan({ platform: "linux", target, tauriDir });
    const calls = [];
    await prepareBeaverCli({
      platform: "linux",
      target,
      tauriDir,
      run: async (spec) => {
        calls.push(spec);
        await mkdir(dirname(plan.source), { recursive: true });
        await writeFile(plan.source, "beaver cli");
      },
    });

    assert.deepEqual(calls, [{ command: "cargo", args: plan.cargoArgs, cwd: tauriDir }]);
    assert.equal(await readFile(plan.destination, "utf8"), "beaver cli");
  } finally {
    await rm(tauriDir, { recursive: true, force: true });
  }
});

test("refuse une cible invalide", () => {
  assert.throws(
    () => createBeaverCliBuildPlan({ platform: "linux", target: "../target", tauriDir: process.cwd() }),
    /Beaver CLI preparation failed/u,
  );
});
