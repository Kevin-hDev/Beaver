const SCRIPT_FONTS = Object.freeze([
  ["emoji", /[\p{Emoji_Presentation}\p{Extended_Pictographic}\p{Regional_Indicator}]/u],
  ["symbols", /[\u2190-\u21ff]/u],
  ["symbols2", /[\u2700-\u27bf]/u],
  ["arabic", /\p{Script=Arabic}/u],
  ["armenian", /\p{Script=Armenian}/u],
  ["hebrew", /\p{Script=Hebrew}/u],
  ["devanagari", /\p{Script=Devanagari}/u],
  ["bengali", /\p{Script=Bengali}/u],
  ["tamil", /\p{Script=Tamil}/u],
  ["sinhala", /\p{Script=Sinhala}/u],
  ["thai", /\p{Script=Thai}/u],
  ["lao", /\p{Script=Lao}/u],
  ["tibetan", /\p{Script=Tibetan}/u],
  ["myanmar", /\p{Script=Myanmar}/u],
  ["georgian", /\p{Script=Georgian}/u],
  ["ethiopic", /\p{Script=Ethiopic}/u],
  ["cherokee", /\p{Script=Cherokee}/u],
  ["khmer", /\p{Script=Khmer}/u],
  ["cjk", /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Hangul}\p{Script=Bopomofo}]/u],
]);

const FALLBACK_FONT_IDS = Object.freeze([
  "base",
  "symbols",
  "symbols2",
  "arabic",
  "armenian",
  "hebrew",
  "devanagari",
  "bengali",
  "tamil",
  "sinhala",
  "thai",
  "lao",
  "tibetan",
  "myanmar",
  "georgian",
  "ethiopic",
  "cherokee",
  "khmer",
  "cjk",
  "emoji",
]);

const INHERITED_SCRIPT = /\p{Script=Inherited}/u;
const JOIN_OR_VARIATION = /[\u200c\u200d\ufe0e\ufe0f]/u;
const WHITE_SPACE = /\p{White_Space}/u;

export function fontRuns(text, fonts) {
  const runs = [];
  for (const { text: runText, fontId } of plannedRuns(text, fonts)) {
    const font = fonts.get(fontId)?.embedded;
    if (!font) throw new Error("font_unavailable");
    runs.push({ text: runText, font });
  }
  return runs;
}

export function plannedRuns(text, fonts) {
  const runs = [];
  let previousFontId = "base";
  for (const character of text) {
    const fontId = selectFontId(character, previousFontId, fonts);
    if (!fontId) throw unsupportedCharacter(character);
    const previous = runs.at(-1);
    if (previous?.fontId === fontId) previous.text += character;
    else runs.push({ text: character, fontId });
    previousFontId = fontId;
  }
  return runs;
}

export function candidateFontIds(character, previousFontId = "base") {
  if (WHITE_SPACE.test(character)) return ["base"];
  if (JOIN_OR_VARIATION.test(character)) return [previousFontId];
  const preferred = preferredFontId(character, previousFontId);
  return [...new Set([preferred, ...FALLBACK_FONT_IDS])];
}

export function fontSupportsCharacter(font, character) {
  return isCoverageIgnorable(character)
    || font.hasGlyphForCodePoint(character.codePointAt(0));
}

export function isCoverageIgnorable(character) {
  return WHITE_SPACE.test(character) || JOIN_OR_VARIATION.test(character);
}

export function preferredFontId(character, previousFontId = "base") {
  if (INHERITED_SCRIPT.test(character)) return previousFontId;
  for (const [fontId, pattern] of SCRIPT_FONTS) {
    if (pattern.test(character)) return fontId;
  }
  return "base";
}

function selectFontId(character, previousFontId, fonts) {
  for (const id of candidateFontIds(character, previousFontId)) {
    const parsed = fonts.get(id)?.parsed;
    if (parsed && fontSupportsCharacter(parsed, character)) return id;
  }
  return undefined;
}

function unsupportedCharacter(character) {
  const error = new Error("unsupported_character");
  error.code = "unsupported_character";
  error.codePoint = character.codePointAt(0);
  return error;
}
