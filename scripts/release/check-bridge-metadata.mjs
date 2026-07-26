import fs from "node:fs";
import { pathToFileURL } from "node:url";

const MAX_CONFIG_BYTES = 16 * 1024 * 1024;
const INTERNAL_NAME = "cl-go-dash";
const PRODUCT_NAME = "CL-GO";
const IDENTIFIER = "com.clgo.dash";

function fail() {
  throw new Error("invalid bridge metadata");
}

function isRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function bridgeVersion(tagValue) {
  if (
    typeof tagValue !== "string" ||
    !/^v1\.0\.([2-9]|[1-9][0-9]{1,2})$/.test(tagValue)
  ) {
    fail();
  }
  return tagValue.slice(1);
}

function cargoPackageVersion(text, header) {
  if (typeof text !== "string") fail();
  const normalizedText = text.replaceAll("\r\n", "\n");
  const marker = `${header}\n`;
  const start = normalizedText.indexOf(marker);
  if (start === -1) fail();
  const bodyStart = start + marker.length;
  const nextSection = normalizedText.indexOf("\n[", bodyStart);
  const block = normalizedText.slice(
    bodyStart,
    nextSection === -1 ? normalizedText.length : nextSection,
  );
  const name = block.match(/^name\s*=\s*"([^"]+)"\s*$/m)?.[1];
  const version = block.match(/^version\s*=\s*"([^"]+)"\s*$/m)?.[1];
  if (name !== INTERNAL_NAME || !version) fail();
  return version;
}

function lockedPackageVersion(text) {
  if (typeof text !== "string") fail();
  const pattern =
    /(?:^|\n)\[\[package\]\]\r?\nname = "cl-go-dash"\r?\nversion = "([^"]+)"/g;
  const first = pattern.exec(text);
  if (!first || pattern.exec(text)) fail();
  return first[1];
}

export function validateBridgeMetadata(metadata, tagValue) {
  if (!isRecord(metadata)) fail();
  const version = bridgeVersion(tagValue);
  const { packageJson, packageLock, cargoToml, cargoLock, tauriConfig } = metadata;

  if (
    !isRecord(packageJson) ||
    packageJson.name !== INTERNAL_NAME ||
    packageJson.version !== version ||
    !isRecord(packageLock) ||
    packageLock.name !== INTERNAL_NAME ||
    packageLock.version !== version ||
    !isRecord(packageLock.packages) ||
    !isRecord(packageLock.packages[""]) ||
    packageLock.packages[""].name !== INTERNAL_NAME ||
    packageLock.packages[""].version !== version ||
    cargoPackageVersion(cargoToml, "[package]") !== version ||
    lockedPackageVersion(cargoLock) !== version ||
    !isRecord(tauriConfig) ||
    tauriConfig.productName !== PRODUCT_NAME ||
    tauriConfig.version !== version ||
    tauriConfig.identifier !== IDENTIFIER
  ) {
    fail();
  }

  return { tag: tagValue, version };
}

function readBoundedText(path) {
  const stat = fs.lstatSync(path);
  if (!stat.isFile() || stat.isSymbolicLink() || stat.size < 1 || stat.size > MAX_CONFIG_BYTES) {
    fail();
  }
  return fs.readFileSync(path, "utf8");
}

function readJson(path) {
  const value = JSON.parse(readBoundedText(path));
  if (!isRecord(value)) fail();
  return value;
}

export function loadBridgeMetadata() {
  return {
    packageJson: readJson("package.json"),
    packageLock: readJson("package-lock.json"),
    cargoToml: readBoundedText("src-tauri/Cargo.toml"),
    cargoLock: readBoundedText("src-tauri/Cargo.lock"),
    tauriConfig: readJson("src-tauri/tauri.conf.json"),
  };
}

function isMainModule() {
  return Boolean(process.argv[1]) && pathToFileURL(process.argv[1]).href === import.meta.url;
}

if (isMainModule()) {
  try {
    if (process.argv.length !== 3) fail();
    validateBridgeMetadata(loadBridgeMetadata(), process.argv[2]);
  } catch {
    console.error("bridge metadata invalid");
    process.exitCode = 1;
  }
}
