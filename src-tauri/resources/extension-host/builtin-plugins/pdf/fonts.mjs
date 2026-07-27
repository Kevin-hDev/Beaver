import { readFile } from "node:fs/promises";
import { fontkit } from "../common/formats/pdf.mjs";

const CJK_FONT = new URL(
  "../common/fonts/NotoSansCJKjp-Regular.otf",
  import.meta.url,
);
const ARABIC_FONT = new URL(
  "../common/fonts/NotoSansArabic-Regular.ttf",
  import.meta.url,
);

export async function embedPdfFonts(document) {
  document.registerFontkit(fontkit);
  const [cjkBytes, arabicBytes] = await Promise.all([
    readFile(CJK_FONT),
    readFile(ARABIC_FONT),
  ]);
  try {
    const [cjk, arabic] = await Promise.all([
      document.embedFont(cjkBytes, { subset: true }),
      document.embedFont(arabicBytes, { subset: true }),
    ]);
    return Object.freeze({ cjk, arabic });
  } finally {
    cjkBytes.fill(0);
    arabicBytes.fill(0);
  }
}

export function fontRuns(text, fonts) {
  const runs = [];
  let kind = "cjk";
  let value = "";
  for (const character of text) {
    const nextKind = fontKind(character, kind);
    if (nextKind !== kind && value) {
      runs.push({ text: value, font: fonts[kind] });
      value = "";
    }
    kind = nextKind;
    value += character;
  }
  if (value) runs.push({ text: value, font: fonts[kind] });
  return runs;
}

function fontKind(character, fallback) {
  const codePoint = character.codePointAt(0);
  if (codePoint === 0x200c || codePoint === 0x200d) return fallback;
  if (
    (codePoint >= 0x0600 && codePoint <= 0x06ff)
    || (codePoint >= 0x0750 && codePoint <= 0x077f)
    || (codePoint >= 0x0870 && codePoint <= 0x089f)
    || (codePoint >= 0x08a0 && codePoint <= 0x08ff)
    || (codePoint >= 0xfb50 && codePoint <= 0xfdff)
    || (codePoint >= 0xfe70 && codePoint <= 0xfeff)
    || (codePoint >= 0x10e60 && codePoint <= 0x10e7f)
    || (codePoint >= 0x1ee00 && codePoint <= 0x1eeff)
  ) {
    return "arabic";
  }
  return "cjk";
}
