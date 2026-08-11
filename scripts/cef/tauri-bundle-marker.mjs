import { lstat, open, realpath } from "node:fs/promises";
import { isAbsolute, resolve, sep } from "node:path";

import { isDirectExecution } from "./direct-execution.mjs";

const ERROR_MESSAGE = "Tauri bundle marker validation failed";
const MAX_BINARY_BYTES = 512 * 1024 * 1024;

export const BUNDLE_MARKERS = Object.freeze({
  unknown: "__TAURI_BUNDLE_TYPE_VAR_UNK",
  msi: "__TAURI_BUNDLE_TYPE_VAR_MSI",
  nsis: "__TAURI_BUNDLE_TYPE_VAR_NSS",
});

export async function prepareTauriBootstrapMarker(path) {
  const binary = await openTrustedBinary(path, "r+");
  try {
    const data = await binary.handle.readFile();
    if (Object.values(BUNDLE_MARKERS).some((marker) => occurrences(data, marker) !== 0)) {
      fail();
    }
    const marker = Buffer.from(BUNDLE_MARKERS.unknown, "ascii");
    if (binary.size + marker.length > MAX_BINARY_BYTES) fail();
    const write = await binary.handle.write(marker, 0, marker.length, binary.size);
    if (write.bytesWritten !== marker.length) fail();
    await binary.handle.sync();
    const verification = Buffer.alloc(marker.length);
    const read = await binary.handle.read(verification, 0, marker.length, binary.size);
    if (read.bytesRead !== marker.length || !verification.equals(marker)) fail();
  } finally {
    await binary.handle.close();
  }
}

export async function patchTauriModuleMarker(path, bundleType) {
  const expected = expectedMarker(bundleType);
  const binary = await openTrustedBinary(path, "r+");
  try {
    const data = await binary.handle.readFile();
    const indexes = markerIndexes(data, BUNDLE_MARKERS.unknown);
    if (indexes.length !== 1) fail();
    const replacement = Buffer.from(expected, "ascii");
    const write = await binary.handle.write(replacement, 0, replacement.length, indexes[0]);
    if (write.bytesWritten !== replacement.length) fail();
    await binary.handle.sync();
    const verification = Buffer.alloc(replacement.length);
    const read = await binary.handle.read(verification, 0, replacement.length, indexes[0]);
    if (read.bytesRead !== replacement.length || !verification.equals(replacement)) fail();
  } finally {
    await binary.handle.close();
  }
}

export async function verifyTauriBundleMarkers({
  bootstrap,
  bundleType,
  module,
  referenceBootstrap,
  referenceModule,
}) {
  const expected = expectedMarker(bundleType);
  await verifyPatchedPair(referenceBootstrap, bootstrap, expected);
  await verifyPatchedPair(referenceModule, module, expected);
}

function expectedMarker(bundleType) {
  if (bundleType !== "msi" && bundleType !== "nsis") fail();
  const marker = BUNDLE_MARKERS[bundleType];
  if (marker.length !== BUNDLE_MARKERS.unknown.length) fail();
  return marker;
}

async function readTrustedBinary(path) {
  const binary = await openTrustedBinary(path, "r");
  try {
    return await binary.handle.readFile();
  } finally {
    await binary.handle.close();
  }
}

async function verifyPatchedPair(referencePath, packagedPath, expected) {
  const [reference, packaged] = await Promise.all([
    readTrustedBinary(referencePath),
    readTrustedBinary(packagedPath),
  ]);
  const indexes = markerIndexes(reference, BUNDLE_MARKERS.unknown);
  if (indexes.length !== 1 || packaged.length !== reference.length) fail();
  const written = reference.write(
    expected,
    indexes[0],
    BUNDLE_MARKERS.unknown.length,
    "ascii",
  );
  if (written !== BUNDLE_MARKERS.unknown.length) fail();
  if (!reference.equals(packaged)) fail();
}

async function openTrustedBinary(path, flags) {
  let handle;
  try {
    if (!safeAbsolutePath(path)) fail();
    const requested = resolve(path);
    handle = await open(requested, flags);
    const opened = await handle.stat();
    const [canonical, current] = await Promise.all([
      realpath(requested),
      lstat(requested),
    ]);
    if (
      comparable(canonical) !== comparable(requested) ||
      !opened.isFile() ||
      !current.isFile() ||
      current.isSymbolicLink() ||
      !sameFile(current, opened) ||
      current.size !== opened.size ||
      (flags !== "r" && opened.nlink > 1) ||
      opened.size < 1 ||
      opened.size > MAX_BINARY_BYTES
    ) {
      fail();
    }
    return { handle, size: opened.size };
  } catch {
    await handle?.close().catch(() => {});
    fail();
  }
}

function safeAbsolutePath(path) {
  return (
    typeof path === "string" &&
    path.length > 0 &&
    path.length <= 30_000 &&
    isAbsolute(path) &&
    !/[\0\r\n]/u.test(path) &&
    !path.split(/[\\/]/u).includes("..")
  );
}

function comparable(path) {
  const value = path.replaceAll("/", sep);
  return process.platform === "win32" ? value.toLowerCase() : value;
}

function sameFile(left, right) {
  return left.dev === right.dev && left.ino === right.ino;
}

function markerIndexes(data, marker) {
  const needle = Buffer.from(marker, "ascii");
  const indexes = [];
  for (let index = data.indexOf(needle); index >= 0; index = data.indexOf(needle, index + 1)) {
    indexes.push(index);
    if (indexes.length > 1) break;
  }
  return indexes;
}

function occurrences(data, marker) {
  return markerIndexes(data, marker).length;
}

function fail() {
  throw new Error(ERROR_MESSAGE);
}

async function main([operation, bundleType, first, second, third, fourth, extra]) {
  if (operation === "prepare-bootstrap" && bundleType && !first && !second && !third) {
    return prepareTauriBootstrapMarker(bundleType);
  }
  if (operation === "patch-module" && bundleType && first && !second && !third) {
    return patchTauriModuleMarker(first, bundleType);
  }
  if (operation === "verify" && bundleType && first && second && third && fourth && !extra) {
    return verifyTauriBundleMarkers({
      bootstrap: first,
      bundleType,
      module: second,
      referenceBootstrap: third,
      referenceModule: fourth,
    });
  }
  fail();
}

if (isDirectExecution(import.meta.url, process.argv[1])) {
  main(process.argv.slice(2)).catch(() => {
    process.stderr.write(`${ERROR_MESSAGE}\n`);
    process.exitCode = 1;
  });
}
