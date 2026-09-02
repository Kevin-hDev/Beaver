import { defineExtension } from "@beaver/sdk";
import { safeTool } from "../common/errors.mjs";
import { createPdf } from "./create.mjs";
import { inspectPdf, mergePdfs } from "./read.mjs";
import {
  createPdfSchema,
  inspectPdfSchema,
  mergePdfSchema,
} from "./schemas.mjs";

export default defineExtension((api) => {
  api.registerTool({
    name: "create",
    effect: "local-write",
    description: "Create a paginated PDF file from a title and paragraphs.",
    parameters: createPdfSchema,
    execute: safeTool(createPdf),
  });
  api.registerTool({
    name: "inspect",
    effect: "read-only",
    description: "Extract bounded text and page information from a PDF file.",
    parameters: inspectPdfSchema,
    execute: safeTool(inspectPdf),
  });
  api.registerTool({
    name: "merge",
    effect: "local-write",
    description: "Merge several PDF files into one bounded PDF document.",
    parameters: mergePdfSchema,
    execute: safeTool(mergePdfs),
  });
});
