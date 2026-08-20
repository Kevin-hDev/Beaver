import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itCatalog from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

describe("subagent read-only translations", () => {
  it("traduit le retour au parent dans les sept catalogues", () => {
    const catalogs = [fr, en, es, de, itCatalog, zh, ja] as unknown as Array<{
      agentLocal: { parentChat?: string };
    }>;

    expect(catalogs.map((catalog) => catalog.agentLocal.parentChat)).toEqual([
      "Chat parent",
      "Parent chat",
      "Conversación principal",
      "Übergeordneter Chat",
      "Chat principale",
      "父级对话",
      "親チャット",
    ]);
  });

  it("utilise la forme allemande Subagenten-Sitzung", () => {
    expect(de.errors.admission.subagentReadOnly).toBe(
      "Diese Subagenten-Sitzung ist schreibgeschützt.",
    );
  });
});
