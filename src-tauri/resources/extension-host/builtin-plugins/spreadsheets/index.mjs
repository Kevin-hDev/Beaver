import { defineExtension } from "@beaver/sdk";
import { safeTool } from "../common/errors.mjs";
import {
  createSpreadsheetSchema,
  inspectSpreadsheetSchema,
  updateSpreadsheetSchema,
} from "./schemas.mjs";
import {
  createSpreadsheet,
  inspectSpreadsheet,
  updateSpreadsheet,
} from "./workbook.mjs";

export default defineExtension((api) => {
  api.registerTool({
    name: "create",
    description: "Create an editable XLSX workbook with one or more sheets.",
    parameters: createSpreadsheetSchema,
    execute: safeTool(createSpreadsheet),
  });
  api.registerTool({
    name: "inspect",
    description: "Inspect bounded cell previews and dimensions from an XLSX workbook.",
    parameters: inspectSpreadsheetSchema,
    execute: safeTool(inspectSpreadsheet),
  });
  api.registerTool({
    name: "update",
    description: "Update selected cells while preserving the existing XLSX workbook.",
    parameters: updateSpreadsheetSchema,
    execute: safeTool(updateSpreadsheet),
  });
});
