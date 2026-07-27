import { OFFICE_LIMITS } from "./constants.mjs";
import { rejectOffice } from "./errors.mjs";

const CENTRAL_SIGNATURE = 0x02014b50;
const END_SIGNATURE = 0x06054b50;
const END_BYTES = 22;
const MAX_COMMENT_BYTES = 65_535;

export function assertSafeOfficeArchive(bytes) {
  if (!Buffer.isBuffer(bytes) || bytes.length < END_BYTES) rejectOffice("unsafe_archive");
  const endOffset = findEnd(bytes);
  const disk = bytes.readUInt16LE(endOffset + 4);
  const centralDisk = bytes.readUInt16LE(endOffset + 6);
  const diskEntries = bytes.readUInt16LE(endOffset + 8);
  const entries = bytes.readUInt16LE(endOffset + 10);
  const centralSize = bytes.readUInt32LE(endOffset + 12);
  const centralOffset = bytes.readUInt32LE(endOffset + 16);
  const commentLength = bytes.readUInt16LE(endOffset + 20);
  if (
    disk !== 0
    || centralDisk !== 0
    || diskEntries !== entries
    || entries === 0
    || entries > OFFICE_LIMITS.maxZipEntries
    || centralOffset + centralSize > endOffset
    || endOffset + END_BYTES + commentLength !== bytes.length
  ) {
    rejectOffice("unsafe_archive");
  }
  inspectEntries(bytes, centralOffset, entries, centralOffset + centralSize);
}

function findEnd(bytes) {
  const minimum = Math.max(0, bytes.length - END_BYTES - MAX_COMMENT_BYTES);
  for (let offset = bytes.length - END_BYTES; offset >= minimum; offset -= 1) {
    if (bytes.readUInt32LE(offset) === END_SIGNATURE) return offset;
  }
  rejectOffice("unsafe_archive");
}

function inspectEntries(bytes, start, count, limit) {
  let offset = start;
  let expandedTotal = 0;
  const names = new Set();
  for (let index = 0; index < count; index += 1) {
    if (offset + 46 > limit || bytes.readUInt32LE(offset) !== CENTRAL_SIGNATURE) {
      rejectOffice("unsafe_archive");
    }
    const flags = bytes.readUInt16LE(offset + 8);
    const compressed = bytes.readUInt32LE(offset + 20);
    const expanded = bytes.readUInt32LE(offset + 24);
    const nameLength = bytes.readUInt16LE(offset + 28);
    const extraLength = bytes.readUInt16LE(offset + 30);
    const commentLength = bytes.readUInt16LE(offset + 32);
    const next = offset + 46 + nameLength + extraLength + commentLength;
    if (
      (flags & 1) !== 0
      || next > limit
      || compressed === 0xffffffff
      || expanded === 0xffffffff
      || expanded > OFFICE_LIMITS.maxZipEntryBytes
      || expanded > Math.max(1_048_576, compressed * OFFICE_LIMITS.maxZipRatio)
    ) {
      rejectOffice("unsafe_archive");
    }
    expandedTotal += expanded;
    if (expandedTotal > OFFICE_LIMITS.maxZipExpandedBytes) {
      rejectOffice("unsafe_archive");
    }
    const name = bytes.toString("utf8", offset + 46, offset + 46 + nameLength);
    if (!safeEntryName(name) || names.has(name)) rejectOffice("unsafe_archive");
    names.add(name);
    offset = next;
  }
  if (offset !== limit) rejectOffice("unsafe_archive");
}

function safeEntryName(name) {
  const parts = name.split(/[\\/]/u);
  return (
    name.length > 0
    && name.length <= 512
    && !name.includes("\0")
    && !name.startsWith("/")
    && !/^[a-zA-Z]:[\\/]/u.test(name)
    && !parts.some((part) => (
      part === ".."
      || part === "__proto__"
      || part === "constructor"
      || part === "prototype"
    ))
  );
}
