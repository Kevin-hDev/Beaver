import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itCatalog from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

function paths(value: unknown, prefix = ""): string[] {
  if (!value || typeof value !== "object") return [prefix];
  return Object.entries(value).flatMap(([key, child]) => paths(child, prefix ? `${prefix}.${key}` : key));
}

describe("traductions des automatisations", () => {
  it("garde le même contrat non vide dans les sept langues", () => {
    const expected = paths(en.heartbeat).sort();
    for (const catalog of [fr, es, de, itCatalog, zh, ja]) {
      expect(paths(catalog.heartbeat).sort()).toEqual(expected);
    }
  });
});
