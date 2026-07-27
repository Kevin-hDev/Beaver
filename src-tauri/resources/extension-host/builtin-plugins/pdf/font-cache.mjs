import { createHash, timingSafeEqual } from "node:crypto";
import { readFile } from "node:fs/promises";
import { PDF_FONT_SPECS } from "../common/fonts/catalog.mjs";
import {
  fontkit,
  logicalFontkit,
} from "../common/formats/pdf.mjs";
import {
  isCoverageIgnorable,
  plannedRuns,
  requiredFontIds,
} from "./font-routing.mjs";

const MAX_CACHED_FONTS = Object.keys(PDF_FONT_SPECS).length;
const cache = new Map();

export async function embedRequiredFonts(document, texts) {
  const ids = requiredFontIds(texts);
  const assets = new Map();
  for (const id of ids) assets.set(id, await fontAsset(id));
  assertCoverage(texts, assets);
  document.registerFontkit(fontkit);
  const visual = await embedAssets(document, assets);
  document.registerFontkit(logicalFontkit);
  const logical = await embedAssets(document, assets);
  document.registerFontkit(fontkit);
  return Object.freeze({ visual, logical });
}

async function embedAssets(document, assets) {
  const embeddedFonts = new Map();
  await Promise.all([...assets].map(async ([id, asset]) => {
    const embedded = await document.embedFont(asset.bytes, { subset: true });
    embeddedFonts.set(id, embedded);
  }));
  return embeddedFonts;
}

async function fontAsset(id) {
  const existing = cache.get(id);
  if (existing) return existing;
  const spec = PDF_FONT_SPECS[id];
  if (!spec || cache.size >= MAX_CACHED_FONTS) throw new Error("font_unavailable");
  const loading = loadFont(spec);
  cache.set(id, loading);
  try {
    return await loading;
  } catch (error) {
    cache.delete(id);
    throw error;
  }
}

async function loadFont(spec) {
  const bytes = await readFile(spec.url);
  try {
    if (bytes.length !== spec.bytes || !matchesSha256(bytes, spec.sha256)) {
      throw new Error("font_integrity_failed");
    }
    return Object.freeze({
      bytes,
      parsed: fontkit.create(bytes),
    });
  } catch (error) {
    bytes.fill(0);
    throw error;
  }
}

function assertCoverage(texts, assets) {
  for (const text of texts) {
    for (const run of plannedRuns(text)) {
      const font = assets.get(run.fontId)?.parsed;
      if (!font) throw new Error("font_unavailable");
      for (const character of run.text) {
        if (
          !isCoverageIgnorable(character)
          && !font.hasGlyphForCodePoint(character.codePointAt(0))
        ) {
          const error = new Error("unsupported_character");
          error.code = "unsupported_character";
          throw error;
        }
      }
    }
  }
}

function matchesSha256(bytes, expectedHex) {
  const actual = createHash("sha256").update(bytes).digest();
  const expected = Buffer.from(expectedHex, "hex");
  try {
    return actual.length === expected.length && timingSafeEqual(actual, expected);
  } finally {
    actual.fill(0);
    expected.fill(0);
  }
}
