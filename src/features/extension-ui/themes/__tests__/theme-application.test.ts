import { describe, expect, it } from "vitest";
import type { ExtensionThemeCatalog, ExtensionThemeEntry } from "../theme-catalog";
import { applyThemeChoice } from "../theme-application";

function entry(
  id: string,
  colorScheme: "light" | "dark",
  tokens: Record<string, string>,
): ExtensionThemeEntry {
  return {
    choice: `extension:${id}`,
    paletteId: id,
    extensionId: "com.example.themes",
    sourceName: "Example Themes",
    label: id,
    colorScheme,
    tokens,
  };
}

function catalog(...entries: ExtensionThemeEntry[]): ExtensionThemeCatalog {
  return {
    ready: true,
    entries,
    byChoice: new Map(entries.map((item) => [item.choice, item])),
  };
}

describe("theme application", () => {
  it.each(["light", "dark"] as const)("hérite de la base %s", (base) => {
    const theme = entry(`com.example.${base}`, base, { "--void": "#010203" });
    const target = document.createElement("div");

    expect(applyThemeChoice(target, theme.choice, catalog(theme), false)).toBe(theme.choice);
    expect(target).toHaveAttribute("data-theme", base);
    expect(target).toHaveAttribute("data-palette", theme.paletteId);
    expect(target.style.getPropertyValue("--void")).toBe("#010203");
  });

  it("retire tous les résidus lors d'un passage tiers, cœur, puis tiers", () => {
    const first = entry("com.example.first", "dark", {
      "--void": "#010203", "--ink": "#ffffff",
    });
    const second = entry("com.example.second", "light", { "--pulse": "#abcdef" });
    const themes = catalog(first, second);
    const target = document.createElement("div");

    applyThemeChoice(target, first.choice, themes, false);
    applyThemeChoice(target, "light", themes, false);
    expect(target.style.getPropertyValue("--void")).toBe("");
    expect(target.style.getPropertyValue("--ink")).toBe("");
    expect(target).toHaveAttribute("data-palette", "light");

    applyThemeChoice(target, second.choice, themes, false);
    expect(target.style.getPropertyValue("--void")).toBe("");
    expect(target.style.getPropertyValue("--pulse")).toBe("#abcdef");
    expect(target).toHaveAttribute("data-theme", "light");
    expect(target).toHaveAttribute("data-palette", "com.example.second");
  });

  it("refuse un choix tiers absent du catalogue", () => {
    const target = document.createElement("div");
    expect(applyThemeChoice(
      target,
      "extension:com.example.missing",
      catalog(),
      false,
    )).toBeNull();
    expect(target).not.toHaveAttribute("data-palette");
  });
});
