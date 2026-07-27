import { PDFDocument, rgb } from "../common/formats/pdf.mjs";
import { OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice } from "../common/errors.mjs";
import { embedRequiredFonts } from "./fonts.mjs";
import { PDF_LAYOUT } from "./layout.mjs";
import { drawTextLine, wrapText } from "./text-layout.mjs";

export async function renderPdf({ title, paragraphs }) {
  const document = await PDFDocument.create();
  document.setCreator("Beaver");
  if (title) document.setTitle(title);
  const fonts = await embedRequiredFonts(
    document,
    title ? [title, ...paragraphs] : paragraphs,
  );
  let page = document.addPage([PDF_LAYOUT.width, PDF_LAYOUT.height]);
  let y = PDF_LAYOUT.height - PDF_LAYOUT.margin;
  if (title) {
    for (const line of wrapText(title, fonts.visual, PDF_LAYOUT.titleSize, textWidth())) {
      ({ page, y } = nextPageIfNeeded(document, page, y, PDF_LAYOUT.titleSize));
      drawTextLine(page, line, fonts, textOptions(y, PDF_LAYOUT.titleSize));
      y -= PDF_LAYOUT.titleSize + PDF_LAYOUT.paragraphGap;
    }
    y -= PDF_LAYOUT.paragraphGap;
  }
  for (const paragraph of paragraphs) {
    for (const line of wrapText(paragraph, fonts.visual, PDF_LAYOUT.bodySize, textWidth())) {
      ({ page, y } = nextPageIfNeeded(document, page, y, PDF_LAYOUT.lineHeight));
      drawTextLine(page, line, fonts, textOptions(y, PDF_LAYOUT.bodySize));
      y -= PDF_LAYOUT.lineHeight;
    }
    y -= PDF_LAYOUT.paragraphGap;
  }
  const pages = document.getPageCount();
  if (pages > OFFICE_LIMITS.maxPdfPages) rejectOffice("output_too_large");
  const bytes = await document.save({ useObjectStreams: true });
  return { bytes, pages };
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
