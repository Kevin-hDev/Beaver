import { describe, expect, it } from "vitest";
import type { StandardCatalogSnapshot } from "../../standard/types";
import { buildThemeCatalog } from "../theme-catalog";

function snapshot(): StandardCatalogSnapshot {
  return {
    revision: 1,
    contributions: [
      {
        extensionId: "com.example.zulu",
        contributionId: "com.example.zulu.blue",
        contribution: {
          type: "theme",
          id: "com.example.zulu.blue",
          order: 0,
          label: { default: "Zulu", fr: "Bleu" },
          base: "dark",
          tokens: { "--pulse": "#010203" },
        },
      },
      {
        extensionId: "com.example.alpha",
        contributionId: "com.example.alpha.red",
        contribution: {
          type: "theme",
          id: "com.example.alpha.red",
          order: 0,
          label: { default: "Alpha", fr: "Ambre" },
          base: "light",
          tokens: {},
        },
      },
    ],
  };
}

describe("theme catalog", () => {
  it("attend le catalogue standard puis trie les thèmes localisés avec leur source", () => {
    expect(buildThemeCatalog(null, new Map(), "fr", false).ready).toBe(false);
    const names = new Map([
      ["com.example.zulu", "Zulu Pack"],
      ["com.example.alpha", "Alpha Pack"],
    ]);
    const catalog = buildThemeCatalog(snapshot(), names, "fr", true);

    expect(catalog.entries.map(({ label }) => label)).toEqual(["Ambre", "Bleu"]);
    expect(catalog.entries.map(({ sourceName }) => sourceName))
      .toEqual(["Alpha Pack", "Zulu Pack"]);
    expect(catalog.byChoice.get("extension:com.example.zulu.blue")?.tokens)
      .toEqual({ "--pulse": "#010203" });
  });
});
