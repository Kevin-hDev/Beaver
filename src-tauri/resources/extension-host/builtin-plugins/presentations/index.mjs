import { defineExtension } from "@beaver/sdk";
import { safeTool } from "../common/errors.mjs";
import { createPresentation } from "./create.mjs";
import { patchPresentation } from "./patch.mjs";
import {
  createPresentationSchema,
  patchPresentationSchema,
} from "./schemas.mjs";

export default defineExtension((api) => {
  api.registerTool({
    name: "create",
    effect: "local-write",
    description: "Create an editable widescreen Microsoft PowerPoint PPTX presentation.",
    parameters: createPresentationSchema,
    execute: safeTool(createPresentation),
  });
  api.registerTool({
    name: "patch",
    effect: "local-write",
    description: "Replace named placeholders in an existing Microsoft PowerPoint PPTX template.",
    parameters: patchPresentationSchema,
    execute: safeTool(patchPresentation),
  });
});
