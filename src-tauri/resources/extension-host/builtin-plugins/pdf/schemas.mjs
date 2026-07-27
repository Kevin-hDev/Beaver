export const createPdfSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: 1_024 },
    title: { type: "string", minLength: 1, maxLength: 300 },
    paragraphs: {
      type: "array",
      minItems: 1,
      maxItems: 500,
      items: { type: "string", minLength: 1, maxLength: 32_767 },
    },
  },
  required: ["path", "paragraphs"],
  additionalProperties: false,
};

export const inspectPdfSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: 1_024 },
    maxPages: { type: "integer", minimum: 1, maximum: 200 },
  },
  required: ["path"],
  additionalProperties: false,
};

export const mergePdfSchema = {
  type: "object",
  properties: {
    sourcePaths: {
      type: "array",
      minItems: 1,
      maxItems: 32,
      items: { type: "string", minLength: 1, maxLength: 1_024 },
    },
    outputPath: { type: "string", minLength: 1, maxLength: 1_024 },
  },
  required: ["sourcePaths", "outputPath"],
  additionalProperties: false,
};
