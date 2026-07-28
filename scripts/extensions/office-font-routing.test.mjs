import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import * as fontkit from "fontkit";
import { PDF_FONT_SPECS } from "../../src-tauri/resources/extension-host/builtin-plugins/common/fonts/catalog.mjs";
import {
  candidateFontIds,
  isCoverageIgnorable,
  preferredFontId,
} from "../../src-tauri/resources/extension-host/builtin-plugins/pdf/font-routing.mjs";

const BLOCKS = Object.freeze([
  [0x00a0, 0x00ff],
  [0x2000, 0x206f],
  [0x2150, 0x218f],
  [0x2190, 0x21ff],
  [0x2200, 0x22ff],
  [0x2300, 0x23ff],
  [0x2460, 0x24ff],
  [0x2500, 0x257f],
  [0x2580, 0x259f],
  [0x25a0, 0x25ff],
  [0x2600, 0x26ff],
  [0x2700, 0x27bf],
]);

test("routes every bundled glyph in common symbol blocks", async () => {
  const fonts = new Map();
  const buffers = [];
  try {
    for (const [id, spec] of Object.entries(PDF_FONT_SPECS)) {
      const bytes = await readFile(spec.url);
      buffers.push(bytes);
      fonts.set(id, fontkit.create(bytes));
    }
    let covered = 0;
    for (const [start, end] of BLOCKS) {
      for (let codePoint = start; codePoint <= end; codePoint += 1) {
        const character = String.fromCodePoint(codePoint);
        const available = [...fonts.values()].some(
          (font) => font.hasGlyphForCodePoint(codePoint),
        );
        if (!available || isCoverageIgnorable(character)) continue;
        const selected = candidateFontIds(character).find(
          (id) => fonts.get(id)?.hasGlyphForCodePoint(codePoint),
        );
        assert.ok(selected, `U+${codePoint.toString(16).toUpperCase()}`);
        covered += 1;
      }
    }
    assert.ok(covered >= 884);
  } finally {
    for (const bytes of buffers) bytes.fill(0);
  }
});

test("keeps common characters out of script-specific routing", () => {
  assert.equal(preferredFontId("·"), "base");
  assert.equal(isCoverageIgnorable("\t"), true);
  assert.deepEqual(candidateFontIds("\t", "arabic"), ["base"]);
  const mathCandidates = candidateFontIds("≤");
  assert.deepEqual(mathCandidates.slice(0, 3), ["base", "symbols", "symbols2"]);
  assert.ok(mathCandidates.indexOf("cjk") > mathCandidates.indexOf("symbols2"));
});
