import { copyFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

interface BeaverFixtureApi {
  registerTool(tool: unknown): void;
  registerSkill(skill: unknown): void;
  registerResource(resource: unknown): void;
}

const ROOT = dirname(fileURLToPath(import.meta.url));

export default function activate(beaver: BeaverFixtureApi) {
  beaver.registerTool({
    name: "catalog_probe",
    description: "Returns an explicit fixture receipt without keyword discovery.",
    parameters: {
      type: "object",
      properties: { query: { type: "string" } },
      required: ["query"],
      additionalProperties: false,
    },
    effect: "read-only",
    execute: ({ query }: { query: string }) => `Explicit fixture call: ${query}`,
  });
  beaver.registerTool({
    name: "produce_artifacts",
    description: "Writes one text artifact and one image preview in the approved workspace.",
    parameters: { type: "object", properties: {}, additionalProperties: false },
    effect: "local-write",
    async execute(_: unknown, context: { workingDirectory: string }) {
      await writeFile(join(context.workingDirectory, "acceptance-artifact.txt"), "API-P7 artifact\n");
      await copyFile(
        join(ROOT, "resources", "preview.png"),
        join(context.workingDirectory, "acceptance-preview.png"),
      );
      return {
        content: [
          { type: "text", text: "Fixture artifacts are ready." },
          {
            type: "file",
            path: "acceptance-artifact.txt",
            purpose: "artifact",
            displayName: "acceptance-artifact.txt",
          },
          {
            type: "file",
            path: "acceptance-preview.png",
            purpose: "preview",
            displayName: "acceptance-preview.png",
          },
        ],
      };
    },
  });
  beaver.registerSkill({
    id: "reference-skill",
    name: "reference-skill",
    description: "Compétence de référence.",
    path: "SKILL.md",
  });
  beaver.registerResource({
    id: "reference",
    name: "reference",
    description: "Référence texte.",
    type: "text",
    path: "resources/reference.txt",
  });
  beaver.registerResource({
    id: "preview",
    name: "preview",
    description: "Aperçu image.",
    type: "image",
    path: "resources/preview.png",
  });
}
