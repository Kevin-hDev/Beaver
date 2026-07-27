const SCRIPT_FONTS = Object.freeze([
  ["emoji", /[\p{Emoji_Presentation}\p{Extended_Pictographic}\p{Regional_Indicator}]/u],
  ["symbols", /[\u2190-\u21ff]/u],
  ["symbols2", /[\u2700-\u27bf]/u],
  ["arabic", /\p{Script_Extensions=Arabic}/u],
  ["armenian", /\p{Script_Extensions=Armenian}/u],
  ["hebrew", /\p{Script_Extensions=Hebrew}/u],
  ["devanagari", /\p{Script_Extensions=Devanagari}/u],
  ["bengali", /\p{Script_Extensions=Bengali}/u],
  ["tamil", /\p{Script_Extensions=Tamil}/u],
  ["sinhala", /\p{Script_Extensions=Sinhala}/u],
  ["thai", /\p{Script_Extensions=Thai}/u],
  ["lao", /\p{Script_Extensions=Lao}/u],
  ["tibetan", /\p{Script_Extensions=Tibetan}/u],
  ["myanmar", /\p{Script_Extensions=Myanmar}/u],
  ["georgian", /\p{Script_Extensions=Georgian}/u],
  ["ethiopic", /\p{Script_Extensions=Ethiopic}/u],
  ["cherokee", /\p{Script_Extensions=Cherokee}/u],
  ["khmer", /\p{Script_Extensions=Khmer}/u],
  ["cjk", /[\p{Script_Extensions=Han}\p{Script_Extensions=Hiragana}\p{Script_Extensions=Katakana}\p{Script_Extensions=Hangul}\p{Script_Extensions=Bopomofo}]/u],
]);

const GRAPHEMES = new Intl.Segmenter("und", { granularity: "grapheme" });
const INHERITED_FORMAT = /[\u200c\u200d\ufe0e\ufe0f]/u;

export function fontRuns(text, fonts) {
  const runs = [];
  for (const planned of plannedRuns(text)) {
    const font = fonts.get(planned.fontId);
    if (!font) throw new Error("font_unavailable");
    runs.push({ text: planned.text, font });
  }
  return runs;
}

export function plannedRuns(text) {
  const runs = [];
  let previousFontId = "base";
  for (const { segment } of GRAPHEMES.segment(text)) {
    const fontId = fontIdForGrapheme(segment, previousFontId);
    const previous = runs.at(-1);
    if (previous?.fontId === fontId) previous.text += segment;
    else runs.push({ text: segment, fontId });
    previousFontId = fontId;
  }
  return runs;
}

export function requiredFontIds(texts) {
  const ids = new Set();
  for (const text of texts) {
    for (const run of plannedRuns(text)) ids.add(run.fontId);
  }
  return ids;
}

export function isCoverageIgnorable(character) {
  return character === "\n"
    || character === "\r"
    || INHERITED_FORMAT.test(character);
}

function fontIdForGrapheme(grapheme, fallback) {
  for (const character of grapheme) {
    if (INHERITED_FORMAT.test(character)) continue;
    for (const [fontId, pattern] of SCRIPT_FONTS) {
      if (pattern.test(character)) return fontId;
    }
    if (!/\s/u.test(character)) return "base";
  }
  return fallback;
}
