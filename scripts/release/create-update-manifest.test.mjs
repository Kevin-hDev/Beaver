import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm, symlink, truncate, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, join } from "node:path";
import test from "node:test";

import {
  createUpdateManifest,
  isValidAssetSize,
  MAX_UPDATE_ASSET_BYTES,
  normalizeVersion,
  writeUpdateManifest,
} from "./create-update-manifest.mjs";

const VERSION = "1.1.0";
const NAMES = [
  "Beaver_1.1.0_aarch64.dmg",
  "Beaver_1.1.0_amd64.deb",
  "Beaver_1.1.0_x64-setup.exe",
];

async function fixture() {
  const directory = await mkdtemp(join(tmpdir(), "beaver-manifest-"));
  for (const name of NAMES) await writeFile(join(directory, name), `content:${name}`);
  return directory;
}

test("normalise uniquement une version stricte", () => {
  assert.equal(normalizeVersion("v1.1.0"), VERSION);
  for (const invalid of ["", "1.1", "01.1.0", "1.1.0-beta", "../1.1.0"]) {
    assert.throws(() => normalizeVersion(invalid));
  }
  assert.throws(() => normalizeVersion("18446744073709551616.1.0"));
  assert.equal(isValidAssetSize(1), true);
  assert.equal(isValidAssetSize(MAX_UPDATE_ASSET_BYTES), true);
  assert.equal(isValidAssetSize(0), false);
  assert.equal(isValidAssetSize(MAX_UPDATE_ASSET_BYTES + 1), false);
});

test("génère trois empreintes triées et déterministes", async () => {
  const directory = await fixture();
  try {
    const first = await createUpdateManifest(VERSION, directory);
    const second = await createUpdateManifest(VERSION, directory);
    assert.deepEqual(first, second);
    assert.deepEqual(
      first.assets.map(({ name }) => name),
      [...NAMES].sort(),
    );
    for (const asset of first.assets) {
      const body = await readFile(join(directory, asset.name));
      assert.equal(asset.sha256, createHash("sha256").update(body).digest("hex"));
      assert.equal(asset.size, body.length);
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("refuse un asset manquant ou inattendu", async () => {
  const missing = await fixture();
  const unexpected = await fixture();
  try {
    await rm(join(missing, NAMES[0]));
    await writeFile(join(unexpected, "extra.txt"), "unexpected");
    await assert.rejects(createUpdateManifest(VERSION, missing));
    await assert.rejects(createUpdateManifest(VERSION, unexpected));
    await assert.rejects(
      createUpdateManifest(VERSION, `${unexpected}/../${basename(unexpected)}`),
    );
  } finally {
    await rm(missing, { recursive: true, force: true });
    await rm(unexpected, { recursive: true, force: true });
  }
});

test("refuse un fichier vide ou symbolique", async () => {
  const empty = await fixture();
  const symbolic = await fixture();
  try {
    await truncate(join(empty, NAMES[0]), 0);
    await rm(join(symbolic, NAMES[0]));
    await assert.rejects(createUpdateManifest(VERSION, empty));
    try {
      await symlink(join(symbolic, NAMES[1]), join(symbolic, NAMES[0]));
      await assert.rejects(createUpdateManifest(VERSION, symbolic));
    } catch (error) {
      if (!["EPERM", "EACCES"].includes(error?.code)) throw error;
    }
  } finally {
    await rm(empty, { recursive: true, force: true });
    await rm(symbolic, { recursive: true, force: true });
  }
});

test("écrit le manifeste final de façon reproductible", async () => {
  const directory = await fixture();
  try {
    const path = await writeUpdateManifest(`v${VERSION}`, directory);
    const first = await readFile(path, "utf8");
    await writeUpdateManifest(VERSION, directory);
    const second = await readFile(path, "utf8");
    assert.equal(first, second);
    assert.deepEqual(JSON.parse(first), await createUpdateManifest(VERSION, directory));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
