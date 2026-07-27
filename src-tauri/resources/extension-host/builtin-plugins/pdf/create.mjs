import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import {
  boundedArray,
  optionalString,
  requiredString,
} from "../common/validation.mjs";
import { atomicWrite, workspaceOutput } from "../common/workspace.mjs";
import { renderPdfInWorker } from "./worker-client.mjs";

export async function createPdf(arguments_, context) {
  const path = requiredString(arguments_?.path, OFFICE_LIMITS.maxPathChars);
  const title = optionalString(arguments_?.title, 300);
  const paragraphs = boundedArray(arguments_?.paragraphs, OFFICE_LIMITS.maxBlocks)
    .map((paragraph) => requiredString(paragraph, 32_767));
  if (paragraphs.reduce((sum, text) => sum + text.length, 0) > OFFICE_LIMITS.maxTextChars) {
    rejectOffice("invalid_input");
  }
  const output = await workspaceOutput(context, path, OFFICE_EXTENSIONS.pdf);
  const result = await renderPdfInWorker({ title, paragraphs });
  try {
    await atomicWrite(output.path, result.bytes);
  } finally {
    result.bytes.fill(0);
  }
  return success({ path, format: "pdf", pages: result.pages });
}
