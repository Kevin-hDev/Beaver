import { PDFDocument, StandardFonts, rgb } from "pdf-lib";
import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import {
  boundedArray,
  optionalString,
  requiredString,
} from "../common/validation.mjs";
import { atomicWrite, workspaceOutput } from "../common/workspace.mjs";
import { PDF_LAYOUT } from "./layout.mjs";

export async function createPdf(arguments_, context) {
  const path = requiredString(arguments_?.path, OFFICE_LIMITS.maxPathChars);
  const title = optionalString(arguments_?.title, 300);
  const paragraphs = boundedArray(arguments_?.paragraphs, OFFICE_LIMITS.maxBlocks)
    .map((paragraph) => requiredString(paragraph, 32_767));
  if (paragraphs.reduce((sum, text) => sum + text.length, 0) > OFFICE_LIMITS.maxTextChars) {
    rejectOffice("invalid_input");
  }
  const document = await PDFDocument.create();
  document.setCreator("Beaver");
  if (title) document.setTitle(title);
  const regular = await document.embedFont(StandardFonts.Helvetica);
  const bold = await document.embedFont(StandardFonts.HelveticaBold);
  let page = document.addPage([PDF_LAYOUT.width, PDF_LAYOUT.height]);
  let y = PDF_LAYOUT.height - PDF_LAYOUT.margin;
  if (title) {
    page.drawText(title, {
      x: PDF_LAYOUT.margin,
      y: y - PDF_LAYOUT.titleSize,
      size: PDF_LAYOUT.titleSize,
      font: bold,
      color: rgb(0.12, 0.14, 0.18),
    });
    y -= PDF_LAYOUT.titleSize + PDF_LAYOUT.paragraphGap * 2;
  }
  for (const paragraph of paragraphs) {
    for (const line of wrapText(paragraph, regular)) {
      if (y < PDF_LAYOUT.margin + PDF_LAYOUT.lineHeight) {
        page = document.addPage([PDF_LAYOUT.width, PDF_LAYOUT.height]);
        y = PDF_LAYOUT.height - PDF_LAYOUT.margin;
      }
      page.drawText(line, {
        x: PDF_LAYOUT.margin,
        y,
        size: PDF_LAYOUT.bodySize,
        font: regular,
        color: rgb(0.12, 0.14, 0.18),
      });
      y -= PDF_LAYOUT.lineHeight;
    }
    y -= PDF_LAYOUT.paragraphGap;
  }
  if (document.getPageCount() > OFFICE_LIMITS.maxPdfPages) {
    rejectOffice("output_too_large");
  }
  const output = await workspaceOutput(context, path, OFFICE_EXTENSIONS.pdf);
  const bytes = await document.save({ useObjectStreams: true });
  await atomicWrite(output.path, bytes);
  return success({ path, format: "pdf", pages: document.getPageCount() });
}

function wrapText(text, font) {
  const width = PDF_LAYOUT.width - PDF_LAYOUT.margin * 2;
  const lines = [];
  for (const sourceLine of text.split(/\r?\n/u)) {
    let current = "";
    for (const word of sourceLine.split(/\s+/u)) {
      const candidate = current ? `${current} ${word}` : word;
      if (font.widthOfTextAtSize(candidate, PDF_LAYOUT.bodySize) <= width) {
        current = candidate;
      } else {
        if (current) lines.push(current);
        current = word;
      }
    }
    lines.push(current);
  }
  return lines;
}
