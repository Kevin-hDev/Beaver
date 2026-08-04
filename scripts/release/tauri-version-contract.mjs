const ERROR_MESSAGE = "Tauri version contract failed";
const MAX_FILE_BYTES = 2 * 1024 * 1024;
const EXPECTED = Object.freeze({
  cli: "2.11.4",
  tauri: "2.11.4",
  build: "2.6.3",
});

function fail() {
  throw new Error(ERROR_MESSAGE);
}

function boundedText(value) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    Buffer.byteLength(value, "utf8") > MAX_FILE_BYTES ||
    value.includes("\0")
  ) {
    fail();
  }
  return value;
}

function packageManifest(value) {
  if (typeof value === "string") return JSON.parse(boundedText(value));
  if (typeof value !== "object" || value === null || Array.isArray(value)) fail();
  return value;
}

function exactManifestVersions(cargoToml) {
  const buildPattern = /^tauri-build\s*=\s*\{\s*version\s*=\s*"=2\.6\.3"\s*,/mu;
  const tauriPattern = /^tauri\s*=\s*\{\s*version\s*=\s*"=2\.11\.4"\s*,/mu;
  return buildPattern.test(cargoToml) && tauriPattern.test(cargoToml);
}

function lockedVersions(cargoLock) {
  const found = { tauri: [], "tauri-build": [] };
  const pattern = /^\[\[package\]\]\r?\nname = "(tauri|tauri-build)"\r?\nversion = "([^"\r\n]{1,32})"/gmu;
  for (const match of cargoLock.matchAll(pattern)) {
    const versions = found[match[1]];
    if (versions.length >= 2) fail();
    versions.push(match[2]);
  }
  return found;
}

export function assertTauriVersionContract({ packageJson, cargoToml, cargoLock } = {}) {
  try {
    const manifestContent = boundedText(cargoToml);
    const lockContent = boundedText(cargoLock);
    const parsedPackage = packageManifest(packageJson);
    if (
      typeof parsedPackage !== "object" ||
      parsedPackage === null ||
      parsedPackage.devDependencies?.["@tauri-apps/cli"] !== EXPECTED.cli ||
      !exactManifestVersions(manifestContent)
    ) {
      fail();
    }
    const locked = lockedVersions(lockContent);
    if (
      locked.tauri.length !== 1 ||
      locked.tauri[0] !== EXPECTED.tauri ||
      locked["tauri-build"].length !== 1 ||
      locked["tauri-build"][0] !== EXPECTED.build
    ) {
      fail();
    }
  } catch {
    fail();
  }
}
