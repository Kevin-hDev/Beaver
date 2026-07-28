import { OFFICE_LIMITS } from "../common/constants.mjs";

const scalar = {
  anyOf: [
    { type: "string", maxLength: 32_767 },
    { type: "number" },
    { type: "boolean" },
    { type: "null" },
  ],
};

export const createSpreadsheetSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    sheets: {
      type: "array",
      minItems: 1,
      maxItems: OFFICE_LIMITS.maxSheets,
      items: {
        type: "object",
        properties: {
          name: { type: "string", minLength: 1, maxLength: 31 },
          rows: {
            type: "array",
            maxItems: OFFICE_LIMITS.maxRowsPerSheet,
            items: {
              type: "array",
              maxItems: OFFICE_LIMITS.maxColumns,
              items: scalar,
            },
          },
        },
        required: ["name", "rows"],
        additionalProperties: false,
      },
    },
  },
  required: ["path", "sheets"],
  additionalProperties: false,
};

export const inspectSpreadsheetSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    maxRows: { type: "integer", minimum: 1, maximum: 200 },
    maxColumns: { type: "integer", minimum: 1, maximum: 100 },
  },
  required: ["path"],
  additionalProperties: false,
};

export const updateSpreadsheetSchema = {
  type: "object",
  properties: {
    sourcePath: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    outputPath: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    changes: {
      type: "array",
      minItems: 1,
      maxItems: OFFICE_LIMITS.maxChanges,
      items: {
        type: "object",
        properties: {
          sheet: { type: "string", minLength: 1, maxLength: 31 },
          cell: {
            type: "string",
            minLength: 2,
            maxLength: 10,
            pattern: "^[A-Za-z]{1,3}[1-9][0-9]{0,6}$",
          },
          value: scalar,
        },
        required: ["sheet", "cell", "value"],
        additionalProperties: false,
      },
    },
  },
  required: ["sourcePath", "outputPath", "changes"],
  additionalProperties: false,
};
