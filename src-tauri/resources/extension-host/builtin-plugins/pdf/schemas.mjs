import { OFFICE_LIMITS } from "../common/constants.mjs";

export const createPdfSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    title: { type: "string", minLength: 1, maxLength: 300 },
    paragraphs: {
      type: "array",
      minItems: 1,
      maxItems: OFFICE_LIMITS.maxBlocks,
      items: { type: "string", minLength: 1, maxLength: 32_767 },
    },
  },
  required: ["path", "paragraphs"],
  additionalProperties: false,
};

export const inspectPdfSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    maxPages: { type: "integer", minimum: 1, maximum: OFFICE_LIMITS.maxPdfPages },
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
      maxItems: OFFICE_LIMITS.maxPdfSources,
      items: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    },
    outputPath: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
  },
  required: ["sourcePaths", "outputPath"],
  additionalProperties: false,
};
