import { LIMITS } from "../../contract.mjs";

const PREVIEW_CHANNEL_FRACTION = 3 / 8;
const STRUCTURED_CHANNEL_FRACTION = 7 / 16;

export const OFFICE_LIMITS = Object.freeze({
  maxInputBytes: 25 * 1024 * 1024,
  maxOutputBytes: 50 * 1024 * 1024,
  maxPathChars: LIMITS.maxWorkingDirectoryChars,
  maxPreviewBytes: Math.floor(
    LIMITS.maxMessageBytes * PREVIEW_CHANNEL_FRACTION,
  ),
  maxStructuredResultBytes: Math.floor(
    LIMITS.maxMessageBytes * STRUCTURED_CHANNEL_FRACTION,
  ),
  maxTextChars: 500_000,
  maxBlocks: 500,
  maxPdfPages: 200,
  maxPdfSources: 32,
  maxSheets: 32,
  maxRowsPerSheet: 10_000,
  maxColumns: 256,
  maxCells: 100_000,
  maxChanges: 5_000,
  maxSlides: 100,
  maxZipEntries: 4_096,
  maxZipExpandedBytes: 128 * 1024 * 1024,
  maxZipEntryBytes: 64 * 1024 * 1024,
  maxZipRatio: 500,
});

export const OFFICE_EXTENSIONS = Object.freeze({
  document: Object.freeze([".docx"]),
  pdf: Object.freeze([".pdf"]),
  spreadsheet: Object.freeze([".xlsx"]),
  presentation: Object.freeze([".pptx"]),
});
