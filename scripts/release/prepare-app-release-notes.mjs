import {
  existsSync,
  lstatSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { randomBytes } from "node:crypto";
import { isDeepStrictEqual } from "node:util";

const ACTIVE_PATH = "app-release-notes.json";
const ARCHIVE_PATH = "app-release-notes-archive.json";
const MAX_SOURCE_BYTES = 2 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES = 1000;
// The runtime accepts 64 KiB. Keeping a margin makes formatting changes harmless.
const MAX_ACTIVE_BYTES = 60 * 1024;
const STABLE_VERSION = /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u;

class PreparationError extends Error {}

try {
  run(process.argv.slice(2));
} catch (error) {
  if (error instanceof PreparationError) {
    console.error(error.message);
  } else {
    console.error("Release notes preparation failed.");
  }
  process.exitCode = 1;
}

function run(arguments_) {
  const checkOnly = arguments_[1] === "--check";
  if (arguments_.length > 2 || (arguments_[1] && !checkOnly)) {
    fail("Invalid release notes preparation arguments.");
  }

  const version = (arguments_[0] ?? "").replace(/^v/u, "");
  if (!STABLE_VERSION.test(version)) {
    fail("A stable release version is required.");
  }

  const active = readDocument(ACTIVE_PATH, false);
  const archive = readDocument(ARCHIVE_PATH, true);
  const selected = active[version];
  if (!isRecord(selected)) {
    fail("The active release notes do not contain the requested version.");
  }

  if (checkOnly) {
    verifyPrepared(active, archive, version);
  } else {
    prepare(active, archive, version, selected);
  }
}

function prepare(active, archive, version, selected) {
  if (Object.hasOwn(archive, version)) {
    fail("The requested version is already archived.");
  }

  const moved = {};
  for (const [entryVersion, notes] of Object.entries(active)) {
    if (entryVersion === version) continue;
    if (!STABLE_VERSION.test(entryVersion) || !isRecord(notes)) {
      fail("The active release notes contain an invalid entry.");
    }
    if (
      Object.hasOwn(archive, entryVersion) &&
      !isDeepStrictEqual(archive[entryVersion], notes)
    ) {
      fail("The active and archived release notes disagree.");
    }
    if (!Object.hasOwn(archive, entryVersion)) {
      moved[entryVersion] = notes;
    }
  }

  const nextActive = { [version]: selected };
  const nextArchive = { ...moved, ...archive };
  if (Object.keys(nextArchive).length > MAX_ARCHIVE_ENTRIES) {
    fail("The release notes archive contains too many versions.");
  }

  const activeText = serialize(nextActive);
  const archiveText = serialize(nextArchive);
  if (Buffer.byteLength(activeText, "utf8") > MAX_ACTIVE_BYTES) {
    fail("The active release notes exceed the application download budget.");
  }
  if (Buffer.byteLength(archiveText, "utf8") > MAX_SOURCE_BYTES) {
    fail("The release notes archive is too large.");
  }

  writePairAtomically(activeText, archiveText);
}

function verifyPrepared(active, archive, version) {
  const activeVersions = Object.keys(active);
  if (
    activeVersions.length !== 1 ||
    activeVersions[0] !== version ||
    Object.hasOwn(archive, version)
  ) {
    fail("The active release notes were not prepared for this tag.");
  }
  if (Buffer.byteLength(serialize(active), "utf8") > MAX_ACTIVE_BYTES) {
    fail("The active release notes exceed the application download budget.");
  }
}

function readDocument(path, optional) {
  if (!existsSync(path)) {
    if (optional) return {};
    fail("The active release notes are missing.");
  }
  if (!lstatSync(path).isFile()) {
    fail("A release notes document is not a regular file.");
  }
  const source = readFileSync(path);
  if (source.length === 0 || source.length > MAX_SOURCE_BYTES) {
    fail("A release notes document has an invalid size.");
  }
  const parsed = JSON.parse(source.toString("utf8"));
  if (!isRecord(parsed) || Object.keys(parsed).length > MAX_ARCHIVE_ENTRIES) {
    fail("A release notes document has an invalid root.");
  }
  return parsed;
}

function writePairAtomically(activeText, archiveText) {
  const suffix = randomBytes(16).toString("hex");
  const activeTemp = `.${ACTIVE_PATH}.${suffix}.tmp`;
  const archiveTemp = `.${ARCHIVE_PATH}.${suffix}.tmp`;
  try {
    writeFileSync(activeTemp, activeText, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
    writeFileSync(archiveTemp, archiveText, {
      encoding: "utf8",
      flag: "wx",
      mode: 0o600,
    });
    // Archive first: if interrupted, rerunning safely recognizes identical entries.
    renameSync(archiveTemp, ARCHIVE_PATH);
    renameSync(activeTemp, ACTIVE_PATH);
  } finally {
    if (existsSync(activeTemp)) unlinkSync(activeTemp);
    if (existsSync(archiveTemp)) unlinkSync(archiveTemp);
  }
}

function serialize(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function isRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function fail(message) {
  throw new PreparationError(message);
}
