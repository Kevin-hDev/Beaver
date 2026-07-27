export const createPresentationSchema = {
  type: "object",
  properties: {
    path: { type: "string", minLength: 1, maxLength: 1_024 },
    title: { type: "string", minLength: 1, maxLength: 300 },
    theme: { type: "string", enum: ["light", "dark"] },
    slides: {
      type: "array",
      minItems: 1,
      maxItems: 100,
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
    sourcePath: { type: "string", minLength: 1, maxLength: 1_024 },
    outputPath: { type: "string", minLength: 1, maxLength: 1_024 },
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
