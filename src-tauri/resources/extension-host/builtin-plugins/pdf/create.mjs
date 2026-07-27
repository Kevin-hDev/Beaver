import { PDFDocument, rgb } from "../common/formats/pdf.mjs";
import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import {
  boundedArray,
  optionalString,
  requiredString,
} from "../common/validation.mjs";
import { atomicWrite, workspaceOutput } from "../common/workspace.mjs";
import { embedPdfFonts } from "./fonts.mjs";
import { PDF_LAYOUT } from "./layout.mjs";
import { drawTextLine, wrapText } from "./text-layout.mjs";

export async function createPdf(arguments_, context) {
  const path = requiredString(arguments_?.path, OFFICE_LIMITS.maxPathChars);
  const title = optionalString(arguments_?.title, 300);
  const paragraphs = boundedArray(arguments_?.paragraphs, OFFICE_LIMITS.maxBlocks)
    .map((paragraph) => requiredString(paragraph, 32_767));
  if (paragraphs.reduce((sum, text) => sum + text.length, 0) > OFFICE_LIMITS.maxTextChars) {
    rejectOffice("invalid_input");
  }
  const output = await workspaceOutput(context, path, OFFICE_EXTENSIONS.pdf);
  const document = await PDFDocument.create();
  document.setCreator("Beaver");
  if (title) document.setTitle(title);
  const fonts = await embedPdfFonts(document);
  let page = document.addPage([PDF_LAYOUT.width, PDF_LAYOUT.height]);
  let y = PDF_LAYOUT.height - PDF_LAYOUT.margin;
  if (title) {
    for (const line of wrapText(title, fonts, PDF_LAYOUT.titleSize, textWidth())) {
      ({ page, y } = nextPageIfNeeded(document, page, y, PDF_LAYOUT.titleSize));
      drawTextLine(page, line, fonts, textOptions(y, PDF_LAYOUT.titleSize));
      y -= PDF_LAYOUT.titleSize + PDF_LAYOUT.paragraphGap;
    }
    y -= PDF_LAYOUT.paragraphGap;
  }
  for (const paragraph of paragraphs) {
    for (const line of wrapText(paragraph, fonts, PDF_LAYOUT.bodySize, textWidth())) {
      ({ page, y } = nextPageIfNeeded(document, page, y, PDF_LAYOUT.lineHeight));
      drawTextLine(page, line, fonts, textOptions(y, PDF_LAYOUT.bodySize));
      y -= PDF_LAYOUT.lineHeight;
    }
    y -= PDF_LAYOUT.paragraphGap;
  }
  if (document.getPageCount() > OFFICE_LIMITS.maxPdfPages) {
    rejectOffice("output_too_large");
  }
  const bytes = await document.save({ useObjectStreams: true });
  await atomicWrite(output.path, bytes);
  return success({ path, format: "pdf", pages: document.getPageCount() });
}

function nextPageIfNeeded(document, page, y, height) {
  if (y >= PDF_LAYOUT.margin + height) return { page, y };
  return {
    page: document.addPage([PDF_LAYOUT.width, PDF_LAYOUT.height]),
    y: PDF_LAYOUT.height - PDF_LAYOUT.margin,
  };
}

function textOptions(y, size) {
  return {
    x: PDF_LAYOUT.margin,
    y,
    size,
    color: rgb(0.12, 0.14, 0.18),
  };
}

function textWidth() {
  return PDF_LAYOUT.width - PDF_LAYOUT.margin * 2;
}
