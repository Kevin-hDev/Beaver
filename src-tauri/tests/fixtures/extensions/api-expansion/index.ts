import { copyFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { BeaverMemoryApi, BeaverModelsApi } from "../../../../resources/extension-host/sdk/core-api";

interface BeaverFixtureApi {
  capabilities?: readonly string[];
  // Share the public SDK result types so the fixture cannot invent another wire shape.
  models?: BeaverModelsApi;
  memory?: BeaverMemoryApi;
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
  const contextualCapabilities = [
    "models",
    "memory",
    "automations",
    "subagents",
    "toolInterception",
  ];
  if (contextualCapabilities.every((capability) => beaver.capabilities?.includes(capability))) {
    beaver.registerTool({
      name: "contextual_journey",
      description: "Runs an attributed model and memory journey and returns a rich receipt.",
      parameters: {
        type: "object",
        properties: { prompt: { type: "string" } },
        required: ["prompt"],
        additionalProperties: false,
      },
      effect: "local-write",
      async execute({ prompt }: { prompt: string }, context: { workingDirectory: string }) {
        const model = await beaver.models!.generate({ prompt });
        const written = await beaver.memory!.write({ scope: "global", content: model.text });
        const reread = await beaver.memory!.read({ scope: "global", topicId: written.topic.id });
        await writeFile(join(context.workingDirectory, "contextual-receipt.txt"), reread.content);
        return {
          content: [
            { type: "text", text: JSON.stringify({ finishReason: model.finishReason, topicId: written.topic.id }) },
            { type: "file", path: "contextual-receipt.txt", purpose: "artifact", displayName: "contextual-receipt.txt" },
          ],
        };
      },
    });
  }
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
