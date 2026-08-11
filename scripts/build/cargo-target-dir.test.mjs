import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, realpath, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve, sep } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";

import {
  canonicalCargoTargetDir,
  normalizeCargoTargetDir,
} from "./cargo-target-dir.mjs";

test("normalise une cible Cargo absolue et bornée", () => {
  const target = resolve("target", ".", "windows");
  assert.equal(normalizeCargoTargetDir(target), target);
});

test("refuse les cibles relatives, traversées ou contenant un contrôle", () => {
  const root = resolve("target");
  for (const target of [
    "target",
    `${root}${sep}..${sep}outside`,
    `${root}${sep}bad\tpath`,
    "x".repeat(30_001),
  ]) {
    assert.throws(
      () => normalizeCargoTargetDir(target),
      /Cargo target validation failed/,
    );
  }
});

test("accepte uniquement un dossier réel non lié", async () => {
  const root = await realpath(
    await mkdtemp(resolve(tmpdir(), "beaver-cargo-target-")),
  );
  const target = resolve(root, "target");
  try {
    await mkdir(target);
    assert.equal(await canonicalCargoTargetDir(target), target);
    await assert.rejects(
      () => canonicalCargoTargetDir(resolve(root, "missing")),
      /Cargo target validation failed/,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("le validateur CLI refuse une autorité Cargo absente", () => {
  const environment = { ...process.env };
  delete environment.CARGO_TARGET_DIR;
  const result = spawnSync(process.execPath, ["scripts/build/cargo-target-dir.mjs"], {
    cwd: process.cwd(),
    env: environment,
    shell: false,
    encoding: "utf8",
  });
  assert.notEqual(result.status, 0);
  assert.equal(result.stderr, "Cargo target validation failed\n");
});

test("le préparateur CEF refuse de recoder le dossier Cargo historique", async () => {
  const script = await readFile("src-tauri/scripts/prepare-cef-windows.ps1", "utf8");
  assert.match(script, /cargo-target-dir\.mjs/u);
  assert.match(script, /\$CargoTargetRoot/u);
  assert.doesNotMatch(script, /Join-Path \$TauriDir "target\\(?:\$BuildTarget\\)?release"/u);
});
