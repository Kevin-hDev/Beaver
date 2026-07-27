import { createHash, timingSafeEqual } from "node:crypto";
import { readFile } from "node:fs/promises";
import { OFFICE_LIMITS } from "../common/constants.mjs";
import { OfficePluginError } from "../common/errors.mjs";
import { PDF_FONT_SPECS } from "../common/fonts/catalog.mjs";
import { fontkit } from "../common/formats/pdf.mjs";
import {
  candidateFontIds,
  fontSupportsCharacter,
} from "./font-routing.mjs";

const cache = new Map();

export async function embedRequiredFonts(document, textEntries) {
  const assets = await resolveAssets(textEntries);
  document.registerFontkit(fontkit);
  const fonts = new Map();
  await Promise.all([...assets].map(async ([id, asset]) => {
    const embedded = await document.embedFont(asset.bytes, { subset: true });
    fonts.set(id, Object.freeze({ embedded, parsed: asset.parsed }));
  }));
  return fonts;
}

async function resolveAssets(textEntries) {
  const loadedAssets = new Map();
  const selectedAssets = new Map();
  for (const entry of textEntries) {
    let previousFontId = "base";
    for (const character of entry.text) {
      const selected = await resolveCharacterFont(
        character,
        previousFontId,
        loadedAssets,
      );
      if (!selected) throw unsupportedCharacter(character, entry);
      selectedAssets.set(selected.id, selected.asset);
      previousFontId = selected.id;
    }
  }
  return selectedAssets;
}

async function resolveCharacterFont(character, previousFontId, loadedAssets) {
  for (const id of candidateFontIds(character, previousFontId)) {
    let asset = loadedAssets.get(id);
    if (!asset) {
      asset = await fontAsset(id);
      loadedAssets.set(id, asset);
    }
    if (fontSupportsCharacter(asset.parsed, character)) {
      return Object.freeze({ id, asset });
    }
  }
  return undefined;
}

async function fontAsset(id) {
  const existing = cache.get(id);
  if (existing) {
    cache.delete(id);
    cache.set(id, existing);
    return existing;
  }
  const spec = PDF_FONT_SPECS[id];
  if (!spec) throw new Error("font_unavailable");
  const loading = loadFont(spec);
  cache.set(id, loading);
  evictOldestFont();
  try {
    return await loading;
  } catch (error) {
    if (cache.get(id) === loading) cache.delete(id);
    throw error;
  }
}

function evictOldestFont() {
  while (cache.size > OFFICE_LIMITS.maxCachedPdfFonts) {
    const oldest = cache.keys().next().value;
    cache.delete(oldest);
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

function unsupportedCharacter(character, entry) {
  return new OfficePluginError("unsupported_character", {
    codePoint: character.codePointAt(0),
    paragraph: entry.paragraph,
    title: entry.title,
  });
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
