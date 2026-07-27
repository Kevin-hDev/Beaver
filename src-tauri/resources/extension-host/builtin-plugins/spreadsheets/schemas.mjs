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
    path: { type: "string", minLength: 1, maxLength: 1_024 },
    sheets: {
      type: "array",
      minItems: 1,
      maxItems: 32,
      items: {
        type: "object",
        properties: {
          name: { type: "string", minLength: 1, maxLength: 31 },
          rows: {
            type: "array",
            maxItems: 10_000,
            items: {
              type: "array",
              maxItems: 256,
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
    path: { type: "string", minLength: 1, maxLength: 1_024 },
    maxRows: { type: "integer", minimum: 1, maximum: 200 },
    maxColumns: { type: "integer", minimum: 1, maximum: 100 },
  },
  required: ["path"],
  additionalProperties: false,
};

export const updateSpreadsheetSchema = {
  type: "object",
  properties: {
    sourcePath: { type: "string", minLength: 1, maxLength: 1_024 },
    outputPath: { type: "string", minLength: 1, maxLength: 1_024 },
    changes: {
      type: "array",
      minItems: 1,
      maxItems: 5_000,
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
