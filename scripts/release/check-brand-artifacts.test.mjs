import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { MAC_CEF_HELPERS } from "../brand/brand-boundaries-platform-contracts.mjs";
import {
  expectedAssetName,
  hashesMatch,
  normalizeVersion,
  validateAsset,
  validateSource,
} from "./brand-artifact-common.mjs";
import {
  validateMacBundle,
  validateReleaseSet,
} from "./check-brand-artifacts.mjs";

const VERSION = "1.1.0";
const PLATFORMS = ["macos", "linux", "windows"];

async function temporaryDirectory() {
  return mkdtemp(join(tmpdir(), "beaver-artifacts-"));
}

async function writeRegular(path, content = "beaver") {
  await mkdir(join(path, ".."), { recursive: true });
  await writeFile(path, content);
}

async function releaseFixture() {
  const directory = await temporaryDirectory();
  const assets = [];
  for (const platform of PLATFORMS) {
    const name = expectedAssetName(platform, VERSION);
    const body = Buffer.from(`asset:${platform}`);
    await writeFile(join(directory, name), body);
    assets.push({
      name,
      sha256: createHash("sha256").update(body).digest("hex"),
      size: body.length,
    });
  }
  assets.sort((left, right) => left.name.localeCompare(right.name));
  await writeFile(
    join(directory, "update-manifest.json"),
    `${JSON.stringify({ version: VERSION, assets })}\n`,
  );
  return directory;
}

async function macFixture() {
  const directory = await temporaryDirectory();
  const app = join(directory, "Beaver.app");
  const dmg = join(directory, expectedAssetName("macos", VERSION));
  const values = new Map();
  const mainPlist = join(app, "Contents/Info.plist");
  await writeRegular(mainPlist, "plist");
  await writeRegular(join(app, "Contents/MacOS/cl-go-dash"));
  await writeRegular(join(app, "Contents/Resources/icon.icns"), "icns-beaver");
  values.set(`${mainPlist}:CFBundleDisplayName`, "Beaver");
  values.set(`${mainPlist}:CFBundleIdentifier`, "com.clgo.dash");
  values.set(`${mainPlist}:CFBundleExecutable`, "cl-go-dash");
  for (const [, helperName, identifier] of MAC_CEF_HELPERS) {
    const helper = join(app, "Contents/Frameworks", helperName);
    const plist = join(helper, "Contents/Info.plist");
    const executable = helperName.slice(0, -4);
    await writeRegular(plist, "plist");
    await writeRegular(join(helper, "Contents/MacOS", executable));
    values.set(`${plist}:CFBundleDisplayName`, executable);
    values.set(`${plist}:CFBundleIdentifier`, identifier);
    values.set(`${plist}:CFBundleExecutable`, executable);
  }
  await writeFile(dmg, "dmg");
  return { directory, app, dmg, plist: (path, key) => values.get(`${path}:${key}`) };
}

test("verrouille la version et les trois noms de fichiers Beaver", () => {
  assert.equal(normalizeVersion("v1.1.0"), VERSION);
  assert.equal(expectedAssetName("macos", VERSION), "Beaver_1.1.0_aarch64.dmg");
  assert.equal(expectedAssetName("linux", VERSION), "Beaver_1.1.0_amd64.deb");
  assert.equal(expectedAssetName("windows", VERSION), "Beaver_1.1.0_x64-setup.exe");
  const hash = "a".repeat(64);
  assert.equal(hashesMatch(hash, hash), true);
  assert.equal(hashesMatch(hash, "b".repeat(64)), false);
  for (const value of [
    "",
    "1.0",
    "01.1.0",
    "1.1.0-beta",
    "../1.1.0",
    "18446744073709551616.0.0",
  ]) {
    assert.throws(() => normalizeVersion(value));
  }
});

test("valide les métadonnées et icônes de la source actuelle", async () => {
  const packageJson = JSON.parse(await readFile("package.json", "utf8"));
  await validateSource(".", `v${packageJson.version}`);
});

test("refuse un asset vide ou mal nommé", async () => {
  const directory = await temporaryDirectory();
  try {
    const valid = join(directory, expectedAssetName("linux", VERSION));
    await writeFile(valid, "deb");
    await validateAsset("linux", VERSION, valid);
    await assert.rejects(validateAsset("windows", VERSION, valid));
    const empty = join(directory, expectedAssetName("windows", VERSION));
    await writeFile(empty, "");
    await assert.rejects(validateAsset("windows", VERSION, empty));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("inspecte Beaver.app et les cinq helpers CEF", async () => {
  const fixture = await macFixture();
  try {
    await validateMacBundle(VERSION, fixture.app, fixture.dmg, fixture.plist);
    await rm(join(fixture.app, "Contents/Frameworks/Beaver Helper (GPU).app"), {
      recursive: true,
    });
    await assert.rejects(
      validateMacBundle(VERSION, fixture.app, fixture.dmg, fixture.plist),
    );
  } finally {
    await rm(fixture.directory, { recursive: true, force: true });
  }
});

test("revérifie indépendamment les tailles et SHA-256 du manifeste", async () => {
  const directory = await releaseFixture();
  try {
    await validateReleaseSet(`v${VERSION}`, directory);
    await writeFile(join(directory, expectedAssetName("linux", VERSION)), "tampered");
    await assert.rejects(validateReleaseSet(VERSION, directory));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("refuse un fichier supplémentaire dans la release", async () => {
  const directory = await releaseFixture();
  try {
    await writeFile(join(directory, "unexpected.txt"), "unexpected");
    await assert.rejects(validateReleaseSet(VERSION, directory));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
