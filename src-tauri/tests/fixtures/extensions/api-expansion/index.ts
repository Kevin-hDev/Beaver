export default function activate(beaver: { registerTool(tool: unknown): void }) {
  beaver.registerTool({
    name: "fixture",
    description: "Outil de fixture API-R0.",
    parameters: { type: "object" },
    effect: "read-only",
    execute: () => "fixture",
  });
}

export const skill = {
  id: "reference-skill",
  name: "reference-skill",
  description: "Compétence de référence.",
  path: "SKILL.md",
};

export const resources = [
  {
    id: "reference",
    name: "reference",
    description: "Référence texte.",
    type: "text",
    path: "resources/reference.txt",
  },
  {
    id: "preview",
    name: "preview",
    description: "Aperçu image.",
    type: "image",
    path: "resources/preview.png",
  },
];
