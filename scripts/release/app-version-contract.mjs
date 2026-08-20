const ERROR_MESSAGE = "Application version contract failed";
const MAX_FILE_BYTES = 2 * 1024 * 1024;
const STABLE_VERSION = /^(0|[1-9][0-9]*)(\.(0|[1-9][0-9]*)){2}$/u;

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

function jsonVersion(source) {
  const parsed = JSON.parse(boundedText(source));
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) fail();
  return parsed.version;
}

function cargoPackageVersion(source) {
  const text = boundedText(source);
  const packageHeading = /^\[package\]\s*$/mu.exec(text);
  if (!packageHeading) fail();
  const tail = text.slice(packageHeading.index + packageHeading[0].length);
  const nextHeading = tail.search(/^\[/mu);
  const section = nextHeading < 0 ? tail : tail.slice(0, nextHeading);
  const versions = [...section.matchAll(/^version\s*=\s*"([^"\r\n]{1,32})"\s*$/gmu)];
  if (versions.length !== 1) fail();
  return versions[0][1];
}

export function assertAppVersionContract({ packageJson, cargoToml, tauriConfig } = {}) {
  try {
    const versions = [
      jsonVersion(packageJson),
      cargoPackageVersion(cargoToml),
      jsonVersion(tauriConfig),
    ];
    if (
      versions.some(
        (version) =>
          typeof version !== "string" || version.length > 32 || !STABLE_VERSION.test(version),
      ) ||
      versions.some((version) => version !== versions[0])
    ) {
      fail();
    }
  } catch {
    fail();
  }
}
