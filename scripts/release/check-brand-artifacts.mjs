import { execFileSync } from "node:child_process";
import { lstat, opendir } from "node:fs/promises";
import { basename, resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { MAC_CEF_HELPERS } from "../brand/brand-boundaries-platform-contracts.mjs";
import {
  ASSET_SUFFIXES,
  expectedAssetName,
  hashFile,
  hashesMatch,
  inside,
  invalid,
  MAX_MANIFEST_BYTES,
  metadata,
  normalizeVersion,
  readJson,
  resolveInputPath,
  validateAsset,
  validateIcon,
  validateSource,
} from "./brand-artifact-common.mjs";

const MAX_ENTRIES = 8;

function nativePlistValue(path, key) {
  if (process.platform !== "darwin") throw invalid();
  try {
    return execFileSync("plutil", ["-extract", key, "raw", "-o", "-", path], {
      encoding: "utf8",
      maxBuffer: 4_096,
      timeout: 10_000,
      windowsHide: true,
    }).trim();
  } catch {
    throw invalid();
  }
}

function nativeDmgHasLicenseAgreement(path) {
  if (process.platform !== "darwin") throw invalid();
  try {
    const imageInfo = execFileSync("hdiutil", ["imageinfo", "-plist", path], {
      maxBuffer: 1024 * 1024,
      timeout: 30_000,
      windowsHide: true,
    });
    const json = execFileSync("plutil", ["-convert", "json", "-o", "-", "-"], {
      input: imageInfo,
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
      timeout: 10_000,
      windowsHide: true,
    });
    return JSON.parse(json)?.Properties?.["Software License Agreement"] === true;
  } catch {
    throw invalid();
  }
}

export async function validateMacBundle(
  versionValue,
  appValue,
  dmgValue,
  plistValue = nativePlistValue,
  dmgHasLicenseAgreement = nativeDmgHasLicenseAgreement,
) {
  await validateAsset("macos", versionValue, dmgValue);
  if (dmgHasLicenseAgreement(dmgValue)) throw invalid();
  const app = resolveInputPath(appValue);
  const appInfo = await lstat(app);
  if (basename(app) !== "Beaver.app" || !appInfo.isDirectory() || appInfo.isSymbolicLink()) {
    throw invalid();
  }
  const mainPlist = inside(app, "Contents/Info.plist");
  if (
    plistValue(mainPlist, "CFBundleDisplayName") !== "Beaver" ||
    plistValue(mainPlist, "CFBundleIdentifier") !== "com.clgo.dash" ||
    plistValue(mainPlist, "CFBundleExecutable") !== "cl-go-dash"
  ) {
    throw invalid();
  }
  await metadata(inside(app, "Contents/MacOS/cl-go-dash"));
  await validateIcon(inside(app, "Contents/Resources"), "icon.icns", "69636e73");
  for (const [, directory, identifier] of MAC_CEF_HELPERS) {
    const helper = inside(app, `Contents/Frameworks/${directory}`);
    const helperInfo = await lstat(helper);
    const plist = inside(helper, "Contents/Info.plist");
    const executable = directory.slice(0, -4);
    if (
      !helperInfo.isDirectory() ||
      helperInfo.isSymbolicLink() ||
      plistValue(plist, "CFBundleDisplayName") !== executable ||
      plistValue(plist, "CFBundleIdentifier") !== identifier ||
      plistValue(plist, "CFBundleExecutable") !== executable
    ) {
      throw invalid();
    }
    await metadata(inside(helper, `Contents/MacOS/${executable}`));
  }
}

export async function validateReleaseSet(versionValue, directoryValue) {
  const version = normalizeVersion(versionValue);
  const directory = resolveInputPath(directoryValue);
  const directoryInfo = await lstat(directory);
  if (!directoryInfo.isDirectory() || directoryInfo.isSymbolicLink()) throw invalid();
  const expected = new Set([
    ...Object.keys(ASSET_SUFFIXES).map((platform) => expectedAssetName(platform, version)),
    "update-manifest.json",
  ]);
  let entries = 0;
  for await (const entry of await opendir(directory)) {
    entries += 1;
    if (entries > MAX_ENTRIES || !entry.isFile() || !expected.delete(entry.name)) {
      throw invalid();
    }
  }
  if (expected.size !== 0 || entries !== 4) throw invalid();
  const manifest = await readJson(inside(directory, "update-manifest.json"), MAX_MANIFEST_BYTES);
  if (
    Object.keys(manifest).sort().join(",") !== "assets,version" ||
    manifest.version !== version ||
    !Array.isArray(manifest.assets) ||
    manifest.assets.length !== 3
  ) {
    throw invalid();
  }
  const seen = new Set();
  for (const asset of manifest.assets) {
    if (
      !asset ||
      Object.keys(asset).sort().join(",") !== "name,sha256,size" ||
      seen.has(asset.name) ||
      !/^[a-f0-9]{64}$/u.test(asset.sha256)
    ) {
      throw invalid();
    }
    seen.add(asset.name);
    const platform = Object.keys(ASSET_SUFFIXES).find(
      (name) => asset.name === expectedAssetName(name, version),
    );
    if (!platform) throw invalid();
    const actual = await hashFile(inside(directory, asset.name));
    if (actual.size !== asset.size || !hashesMatch(actual.sha256, asset.sha256)) {
      throw invalid();
    }
  }
  if (seen.size !== 3) throw invalid();
}

async function main([mode, version, first, second]) {
  if (mode === "source" && first && !second) return validateSource(first, version);
  if (mode === "macos" && first && second) return validateMacBundle(version, first, second);
  if (mode === "release" && first && !second) return validateReleaseSet(version, first);
  if (["linux", "windows"].includes(mode) && first && !second) {
    return validateAsset(mode, version, first);
  }
  throw invalid();
}

const invoked = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invoked) {
  main(process.argv.slice(2))
    .then(() => console.log("Beaver artifacts validated."))
    .catch(() => {
      console.error("Beaver artifact validation failed.");
      process.exitCode = 1;
    });
}
