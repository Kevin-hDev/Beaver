import { OFFICE_LIMITS } from "../common/constants.mjs";

export const createPresentationSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: OFFICE_LIMITS.maxPathChars },
    title: { type: "string", minLength: 1, maxLength: 300 },
    language: {
      type: "string",
      minLength: 2,
      maxLength: 35,
      pattern: "^[A-Za-z]{2,8}(?:-[A-Za-z0-9]{1,8}){0,3}$",
    },
    theme: { type: "string", enum: ["light", "dark"] },
    slides: {
      type: "array",
      minItems: 1,
      maxItems: OFFICE_LIMITS.maxSlides,
      items: {
        type: "object",
        properties: {
          title: { type: "string", minLength: 1, maxLength: 300 },
          bullets: {
            type: "array",
            maxItems: 20,
            items: { type: "string", minLength: 1, maxLength: 2_000 },
          },
          notes: { type: "string", maxLength: 10_000 },
        },
        required: ["title", "bullets"],
        additionalProperties: false,
      },
    },
  },
  required: ["path", "slides"],
  additionalProperties: false,
};

export const patchPresentationSchema = {
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
