import assert from "node:assert/strict";
import {
  lstat,
  link,
  mkdtemp,
  mkdir,
  readFile,
  realpath,
  rm,
  symlink,
  truncate,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";

import {
  copyVerifiedAtomic,
  createUpdaterBuildPlan,
  prepareUpdaterHelper,
} from "./prepare-updater-helper.mjs";

const GENERIC_ERROR = /Updater helper preparation failed/u;

async function withTemporaryDirectory(run) {
  const directory = await mkdtemp(join(await realpath(tmpdir()), "beaver-updater-helper-"));
  try {
    await run(directory);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

test("construit le helper Windows ciblé et sa destination", () => {
  const plan = createUpdaterBuildPlan({
    platform: "win32",
    target: "x86_64-pc-windows-msvc",
    tauriDir: process.cwd(),
  });

  assert.deepEqual(plan.cargoArgs, [
    "build",
    "--release",
    "--bin",
    "cl-go-dash-updater",
    "--target",
    "x86_64-pc-windows-msvc",
  ]);
  assert.match(plan.source, /cl-go-dash-updater\.exe$/u);
  assert.match(plan.destination, /target[\\/]updater-helper[\\/]cl-go-dash-updater\.exe$/u);
});

test("construit le helper Unix sans extension", () => {
  const plan = createUpdaterBuildPlan({
    platform: "linux",
    target: "",
    tauriDir: process.cwd(),
  });

  assert.deepEqual(plan.cargoArgs, ["build", "--release", "--bin", "cl-go-dash-updater"]);
  assert.match(plan.source, /target[\\/]release[\\/]cl-go-dash-updater$/u);
  assert.match(plan.destination, /target[\\/]updater-helper[\\/]cl-go-dash-updater$/u);
});

test("refuse les cibles invalides", () => {
  for (const target of ["x".repeat(129), "windows target", "bad\ntarget", "../target", ".."] ) {
    assert.throws(
      () => createUpdaterBuildPlan({ platform: "win32", target, tauriDir: process.cwd() }),
      GENERIC_ERROR,
    );
  }
});

test("copie le helper et vérifie son contenu", async () => {
  await withTemporaryDirectory(async (directory) => {
    const source = join(directory, "source.exe");
    const destination = join(directory, "target", "updater-helper", "helper.exe");
    await writeFile(source, Buffer.from("verified helper"));

    await copyVerifiedAtomic(source, destination, 64 * 1024 * 1024);

    assert.deepEqual(await readFile(destination), Buffer.from("verified helper"));
    assert.equal((await lstat(destination)).isFile(), true);
  });
});

test("remplace le placeholder Windows existant", async () => {
  await withTemporaryDirectory(async (directory) => {
    const source = join(directory, "source.exe");
    const destination = join(directory, "target", "updater-helper", "helper.exe");
    await mkdir(dirname(destination), { recursive: true });
    await writeFile(source, Buffer.from("compiled helper"));
    await writeFile(destination, Buffer.alloc(0));

    await copyVerifiedAtomic(source, destination, 64 * 1024 * 1024);

    assert.deepEqual(await readFile(destination), Buffer.from("compiled helper"));
  });
});

test("accepte le lien physique borné produit par Cargo", async () => {
  await withTemporaryDirectory(async (directory) => {
    const source = join(directory, "release", "helper.exe");
    const cargoCopy = join(directory, "release", "deps", "helper.exe");
    const destination = join(directory, "updater-helper", "helper.exe");
    await mkdir(dirname(cargoCopy), { recursive: true });
    await writeFile(source, Buffer.from("cargo helper"));
    await link(source, cargoCopy);

    await copyVerifiedAtomic(source, destination, 64 * 1024 * 1024);

    assert.deepEqual(await readFile(destination), Buffer.from("cargo helper"));
  });
});

test("refuse un helper vide ou supérieur à 64 Mio", async () => {
  await withTemporaryDirectory(async (directory) => {
    const empty = join(directory, "empty.exe");
    const oversized = join(directory, "oversized.exe");
    const destination = join(directory, "helper.exe");
    await writeFile(empty, Buffer.alloc(0));
    await writeFile(oversized, Buffer.from([0]));
    await truncate(oversized, 64 * 1024 * 1024 + 1);

    await assert.rejects(() => copyVerifiedAtomic(empty, destination, 64 * 1024 * 1024), GENERIC_ERROR);
    await assert.rejects(() => copyVerifiedAtomic(oversized, destination, 64 * 1024 * 1024), GENERIC_ERROR);
  });
});

test("refuse une source ou destination liée", async () => {
  await withTemporaryDirectory(async (directory) => {
    const realSource = join(directory, "real-source");
    const sourceLink = join(directory, "source-link.exe");
    const destinationTarget = join(directory, "destination-target");
    const destinationLink = join(directory, "destination-link.exe");
    await mkdir(realSource);
    await mkdir(destinationTarget);
    await symlink(realSource, sourceLink, process.platform === "win32" ? "junction" : "dir");
    await symlink(destinationTarget, destinationLink, process.platform === "win32" ? "junction" : "dir");

    await assert.rejects(() => copyVerifiedAtomic(sourceLink, join(directory, "helper.exe"), 1024), GENERIC_ERROR);
    await writeFile(join(directory, "source.exe"), Buffer.from("helper"));
    await assert.rejects(() => copyVerifiedAtomic(join(directory, "source.exe"), destinationLink, 1024), GENERIC_ERROR);
  });
});

test("compile avec cargo puis publie le helper", async () => {
  await withTemporaryDirectory(async (tauriDir) => {
    const calls = [];
    await prepareUpdaterHelper({
      platform: "win32",
      target: "x86_64-pc-windows-msvc",
      tauriDir,
      run: async (spec) => {
        calls.push(spec);
        const plan = createUpdaterBuildPlan({ platform: "win32", target: "x86_64-pc-windows-msvc", tauriDir });
        await mkdir(dirname(plan.source), { recursive: true });
        await writeFile(plan.source, Buffer.from("compiled helper"));
      },
    });

    const plan = createUpdaterBuildPlan({ platform: "win32", target: "x86_64-pc-windows-msvc", tauriDir });
    assert.deepEqual(calls, [{ command: "cargo", args: plan.cargoArgs, cwd: tauriDir }]);
    assert.deepEqual(await readFile(plan.destination), Buffer.from("compiled helper"));
  });
});
