import { describe, expect, it } from "vitest";

/* Le menu d'une conversation était écrit deux fois : une fois pour les
   conversations hors projet, une fois pour celles d'un projet. Les deux copies
   ont divergé — une commande ajoutée d'un côté a manqué de l'autre pendant une
   version. Ce test relit les sources : il n'existe qu'un seul endroit où ces
   commandes sont déclarées, et tout affichage du menu passe par lui. */
const SOURCES: Record<string, string> = import.meta.glob("/src/**/*.{ts,tsx}", { eager: true, query: "?raw", import: "default" });

const AUTHORITY = "/src/components/agent-local/use-session-menu-items.tsx";

function sourcesOutsideTests(): [string, string][] {
  return Object.entries(SOURCES).filter(([path]) => !path.includes("/__tests__/"));
}

describe("autorité unique du menu d'une conversation", () => {
  /* Le repère est la réunion des deux commandes : « archiver » seul désigne
     aussi de simples boutons ailleurs, et n'annonce pas un menu. */
  it("ne déclare les commandes qu'à un seul endroit", () => {
    const declarations = sourcesOutsideTests()
      .filter(([, c]) => c.includes("history.archive") && c.includes("history.rename"))
      .map(([path]) => path);

    expect(declarations).toEqual([AUTHORITY]);
  });

  it("est utilisée par les deux listes qui affichent des conversations", () => {
    const consumers = sourcesOutsideTests()
      .filter(([path, content]) => path !== AUTHORITY && content.includes("useSessionMenuItems"))
      .map(([path]) => path)
      .sort();

    expect(consumers).toEqual([
      "/src/components/agent-local/conversation-list.tsx",
      "/src/components/agent-local/project-section.tsx",
    ]);
  });
});
