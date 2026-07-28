import { defineExtension } from "@beaver/sdk";
import { safeTool } from "../common/errors.mjs";
import { createDocumentSchema, patchDocumentSchema } from "./schemas.mjs";
import { createDocument, patchDocumentTemplate } from "./tools.mjs";

export default defineExtension((api) => {
  api.registerTool({
    name: "create",
    description: "Create an editable Microsoft Word DOCX document from structured text blocks.",
    parameters: createDocumentSchema,
    execute: safeTool(createDocument),
  });
  api.registerTool({
    name: "patch",
    description: "Replace named placeholders in an existing Microsoft Word DOCX template.",
    parameters: patchDocumentSchema,
    execute: safeTool(patchDocumentTemplate),
  });
});
