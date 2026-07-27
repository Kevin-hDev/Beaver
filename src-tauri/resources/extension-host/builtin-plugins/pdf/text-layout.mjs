import { fontRuns } from "./fonts.mjs";

export function wrapText(text, fonts, size, width) {
  const lines = [];
  for (const sourceLine of text.split(/\r?\n/u)) {
    if (!sourceLine.trim()) {
      lines.push("");
      continue;
    }
    wrapSourceLine(sourceLine, fonts, size, width, lines);
  }
  return lines;
}

export function drawTextLine(page, text, fonts, options) {
  let x = options.x;
  for (const run of fontRuns(text, fonts)) {
    page.drawText(run.text, { ...options, x, font: run.font });
    x += run.font.widthOfTextAtSize(run.text, options.size);
  }
}

function wrapSourceLine(source, fonts, size, width, lines) {
  const tokens = source.match(/\s+|\S+/gu) ?? [];
  let current = "";
  for (const token of tokens) {
    const candidate = current + token;
    if (textWidth(candidate, fonts, size) <= width) {
      current = candidate;
      continue;
    }
    if (current.trim()) lines.push(current.trimEnd());
    current = "";
    if (!token.trim()) continue;
    const parts = splitWideToken(token, fonts, size, width);
    lines.push(...parts.slice(0, -1));
    current = parts.at(-1) ?? "";
  }
  if (current.trim()) lines.push(current.trimEnd());
}

function splitWideToken(token, fonts, size, width) {
  const characters = Array.from(token);
  const parts = [];
  let start = 0;
  while (start < characters.length) {
    let minimum = start + 1;
    let maximum = characters.length;
    let accepted = start + 1;
    while (minimum <= maximum) {
      const middle = Math.floor((minimum + maximum) / 2);
      const candidate = characters.slice(start, middle).join("");
      if (textWidth(candidate, fonts, size) <= width) {
        accepted = middle;
        minimum = middle + 1;
      } else {
        maximum = middle - 1;
      }
    }
    parts.push(characters.slice(start, accepted).join(""));
    start = accepted;
  }
  return parts;
}

function textWidth(text, fonts, size) {
  return fontRuns(text, fonts).reduce(
    (width, run) => width + run.font.widthOfTextAtSize(run.text, size),
    0,
  );
}
