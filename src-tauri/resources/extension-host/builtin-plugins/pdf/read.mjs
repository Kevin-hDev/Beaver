import { PDFDocument } from "../common/formats/pdf.mjs";
import { getDocument } from "../common/formats/pdf-reader.mjs";
import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import { createResultBudget } from "../common/result-budget.mjs";
import { boundedArray, requiredString } from "../common/validation.mjs";
import {
  atomicWrite,
  readWorkspaceFile,
  workspaceOutput,
} from "../common/workspace.mjs";

export async function inspectPdf(arguments_, context) {
  const path = requiredString(arguments_?.path, OFFICE_LIMITS.maxPathChars);
  const maxPages = arguments_?.maxPages ?? 20;
  if (!Number.isInteger(maxPages) || maxPages < 1 || maxPages > OFFICE_LIMITS.maxPdfPages) {
    rejectOffice("invalid_input");
  }
  const input = await readWorkspaceFile(context, path, OFFICE_EXTENSIONS.pdf);
  let document;
  let task;
  try {
    task = getDocument({
      data: new Uint8Array(input.bytes),
      disableFontFace: true,
      isEvalSupported: false,
      useSystemFonts: false,
    });
    document = await task.promise;
    if (document.numPages > OFFICE_LIMITS.maxPdfPages) rejectOffice("file_too_large");
    const pages = [];
    const budget = createResultBudget();
    let resultTruncated = false;
    const count = Math.min(document.numPages, maxPages);
    for (let number = 1; number <= count && !resultTruncated; number += 1) {
      const page = await document.getPage(number);
      const structuredText = actualTextItems(await page.getStructTree());
      const content = structuredText
        ? undefined
        : await page.getTextContent();
      const textParts = [];
      const sourceItems = structuredText
        ?? content.items.flatMap((item) =>
          typeof item?.str === "string" ? [item.str] : []);
      const items = sourceItems.slice(0, 10_000);
      if (sourceItems.length > items.length) resultTruncated = true;
      budget.reserve(128);
      for (const item of items) {
        const part = budget.takeText(item, 2);
        if (part.value) textParts.push(part.value);
        if (part.truncated) {
          resultTruncated = true;
          break;
        }
      }
      pages.push({
        page: number,
        text: textParts.join(" "),
        truncated: resultTruncated,
      });
      page.cleanup();
    }
    return success({
      path,
      format: "pdf",
      pageCount: document.numPages,
      truncated: document.numPages > count || resultTruncated,
      pages,
    });
  } finally {
    document?.cleanup();
    await task?.destroy().catch(() => {});
    input.bytes.fill(0);
  }
}

function actualTextItems(tree) {
  if (!tree || tree.role !== "Root" || !Array.isArray(tree.children)) {
    return undefined;
  }
  if (tree.children.length === 0 || tree.children.length > 10_000) {
    return undefined;
  }
  const items = [];
  for (const child of tree.children) {
    if (
      child?.role !== "Span"
      || typeof child.alt !== "string"
      || child.alt.length > 32_767
      || !Array.isArray(child.children)
      || child.children.length !== 1
      || child.children[0]?.type !== "content"
      || typeof child.children[0]?.id !== "string"
      || child.children[0].id.length > 128
    ) {
      return undefined;
    }
    items.push(child.alt);
  }
  return items;
}

export async function mergePdfs(arguments_, context) {
  const sourcePaths = boundedArray(
    arguments_?.sourcePaths,
    OFFICE_LIMITS.maxPdfSources,
  ).map((path) => requiredString(path, OFFICE_LIMITS.maxPathChars));
  const outputPath = requiredString(
    arguments_?.outputPath,
    OFFICE_LIMITS.maxPathChars,
  );
  const output = await workspaceOutput(context, outputPath, OFFICE_EXTENSIONS.pdf);
  const outputDocument = await PDFDocument.create();
  let totalInputBytes = 0;
  for (const sourcePath of sourcePaths) {
    const input = await readWorkspaceFile(context, sourcePath, OFFICE_EXTENSIONS.pdf);
    totalInputBytes += input.bytes.length;
    if (totalInputBytes > OFFICE_LIMITS.maxOutputBytes) {
      input.bytes.fill(0);
      rejectOffice("file_too_large");
    }
    try {
      const source = await PDFDocument.load(input.bytes, {
        ignoreEncryption: false,
        updateMetadata: false,
      });
      if (outputDocument.getPageCount() + source.getPageCount() > OFFICE_LIMITS.maxPdfPages) {
        rejectOffice("file_too_large");
      }
      const pages = await outputDocument.copyPages(source, source.getPageIndices());
      for (const page of pages) outputDocument.addPage(page);
    } finally {
      input.bytes.fill(0);
    }
  }
  const bytes = await outputDocument.save({ useObjectStreams: true });
  await atomicWrite(output.path, bytes);
  return success({
    path: outputPath,
    format: "pdf",
    pages: outputDocument.getPageCount(),
  });
}
