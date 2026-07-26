import fs from "node:fs";

const SUPPORTED_LOCALES = ["fr", "en", "es", "de", "it", "zh", "ja"];
const MAX_BULLETS = 6;
const MAX_BULLET_CHARS = 180;
const MAX_VERSION_ENTRIES = 50;

const version = (process.argv[2] || process.env.RELEASE_VERSION || "").replace(/^v/, "");
if (!version) {
  throw new Error("Missing release version");
}

assertChangelogVersion(version);
const notes = readReleaseNotes(version);
const body = ["### App release notes", "", ...notes.en.map((line) => `- ${line}`)].join("\n");

if (process.env.GITHUB_OUTPUT) {
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `body<<EOF\n${body}\nEOF\n`);
} else {
  console.log(body);
}

function assertChangelogVersion(version) {
  const changelog = fs.readFileSync("CHANGELOG.md", "utf8");
  if (!new RegExp(`^## v${escapeRegExp(version)}$`, "m").test(changelog)) {
    throw new Error(`Missing CHANGELOG.md section for v${version}`);
  }
}

function readReleaseNotes(version) {
  const root = JSON.parse(fs.readFileSync("app-release-notes.json", "utf8"));
  if (!isRecord(root) || Object.keys(root).length > MAX_VERSION_ENTRIES) {
    throw new Error("Invalid app-release-notes.json root");
  }

  const notes = root[version] ?? root[`v${version}`];
  if (!isRecord(notes)) {
    throw new Error(`Missing app-release-notes.json entry for ${version}`);
  }

  const keys = Object.keys(notes).sort();
  const expected = [...SUPPORTED_LOCALES].sort();
  if (keys.join(",") !== expected.join(",")) {
    throw new Error(`Release notes for ${version} must include exactly: ${expected.join(", ")}`);
  }

  for (const locale of SUPPORTED_LOCALES) {
    const items = notes[locale];
    if (!Array.isArray(items) || items.length < 1 || items.length > MAX_BULLETS) {
      throw new Error(`${locale} release notes must contain 1 to ${MAX_BULLETS} items`);
    }
    for (const item of items) {
      validateItem(locale, item);
    }
  }

  return notes;
}

function validateItem(locale, item) {
  if (typeof item !== "string" || !item.trim()) {
    throw new Error(`${locale} release note must be a non-empty string`);
  }
  if (item !== item.trim() || /[\r\n\t]/.test(item)) {
    throw new Error(`${locale} release note has invalid whitespace`);
  }
  if ([...item].length > MAX_BULLET_CHARS) {
    throw new Error(`${locale} release note is too long: ${item}`);
  }
  if (!/[.!?。！？]$/.test(item)) {
    throw new Error(`${locale} release note must be a complete sentence: ${item}`);
  }
}

function isRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
