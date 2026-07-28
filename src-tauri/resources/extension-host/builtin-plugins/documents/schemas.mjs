import { OFFICE_LIMITS } from "../common/constants.mjs";

const block = {
  type: "object",
  properties: {
    type: { type: "string", enum: ["heading", "paragraph", "bullet"] },
    text: { type: "string", minLength: 1, maxLength: 32_767 },
    level: { type: "integer", minimum: 1, maximum: 6 },
  },
  required: ["type", "text"],
  additionalProperties: false,
};

export const createDocumentSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    title: { type: "string", minLength: 1, maxLength: 300 },
    blocks: {
      type: "array",
      minItems: 1,
      maxItems: OFFICE_LIMITS.maxBlocks,
      items: block,
    },
  },
  required: ["path", "blocks"],
  additionalProperties: false,
};

export const patchDocumentSchema = {
  type: "object",
  properties: {
    sourcePath: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    outputPath: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    replacements: {
      type: "object",
      minProperties: 1,
      maxProperties: 256,
      additionalProperties: {
        type: "string",
        maxLength: 32_767,
      },
    },
  },
  required: ["sourcePath", "outputPath", "replacements"],
  additionalProperties: false,
};
