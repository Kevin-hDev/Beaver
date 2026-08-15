import { describe, expect, it } from "vitest";

/* Le glisser-déposer de réordonnancement était écrit deux fois : une fois pour
   les projets de la barre latérale, une fois pour les onglets du terminal. Les
   deux souffraient du même défaut — mesurer la cible pendant que le geste la
   déplace — et il a fallu le trouver deux fois. Ce test relit les sources : le
   geste n'est décrit qu'à un seul endroit, et les listes y passent toutes. */
const SOURCES: Record<string, string> = import.meta.glob("/src/**/*.{ts,tsx}", { eager: true, query: "?raw", import: "default" });

const AUTHORITY = "/src/hooks/use-drag-reorder.ts";

function sourcesOutsideTests(): [string, string][] {
  return Object.entries(SOURCES).filter(([path]) => !path.includes("/__tests__/") && !path.endsWith(".test.ts") && !path.endsWith(".test.tsx"));
}

describe("autorité unique du réordonnancement par glissement", () => {
  it("ne pose la marque des cases déplaçables qu'à un seul endroit", () => {
    const declarations = sourcesOutsideTests()
      .filter(([, content]) => content.includes('"data-drag-id"'))
      .map(([path]) => path);

    expect(declarations).toEqual([AUTHORITY]);
  });

  it("est utilisée par les deux listes qui se réordonnent", () => {
    const consumers = sourcesOutsideTests()
      .filter(([path, content]) => path !== AUTHORITY && content.includes("useDragReorder"))
      .map(([path]) => path)
      .sort();

    expect(consumers).toEqual([
      "/src/components/agent-local/conversation-list.tsx",
      "/src/components/agent-local/project-section.tsx",
      "/src/components/terminal/terminal-tab-bar.tsx",
    ]);
  });
});
