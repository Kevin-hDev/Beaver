interface BeaverFixtureApi {
  registerTool(tool: unknown): void;
  registerSkill(skill: unknown): void;
  registerResource(resource: unknown): void;
}

export default function activate(beaver: BeaverFixtureApi) {
  beaver.registerTool({
    name: "fixture",
    description: "Outil de fixture API-R0.",
    parameters: { type: "object" },
    effect: "read-only",
    execute: () => "fixture",
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
