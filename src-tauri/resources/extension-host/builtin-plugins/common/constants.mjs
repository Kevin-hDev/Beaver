import { LIMITS } from "../../contract.mjs";

const PREVIEW_CHANNEL_FRACTION = 3 / 8;
const STRUCTURED_CHANNEL_FRACTION = 7 / 16;
// Un caractère pèse 1 octet en latin mais 3 en CJK, arabe ou devanagari, et 4 en
// emoji. Sans borne en octets, `maxTextChars` reste inatteignable pour ces
// écritures : la limite du transport tombe la première, avec une erreur qui ne
// désigne pas le texte de l'appelant. La fraction laisse la place à l'enveloppe
// JSON-RPC et au reste des arguments.
const TEXT_CHANNEL_FRACTION = 3 / 4;

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
  maxTextBytes: Math.floor(LIMITS.maxMessageBytes * TEXT_CHANNEL_FRACTION),
  maxBlocks: 500,
  maxPdfPages: 200,
  maxPdfSources: 32,
  maxPdfRenderRequests: 4,
  maxCachedPdfFonts: 8,
  maxPdfTaggedLines: 12_000,
  pdfRenderTimeoutMs: 60_000,
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
