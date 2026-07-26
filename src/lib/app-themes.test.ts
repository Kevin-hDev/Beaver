import { describe, expect, it } from "vitest";
import {
  getNextThemeChoice,
  getThemeColorScheme,
  isThemeChoice,
  resolveTheme,
  THEME_OPTIONS,
} from "./app-themes";

describe("app themes", () => {
  it("déclare Émeraude nocturne comme une palette sombre", () => {
    expect(THEME_OPTIONS).toContainEqual({
      id: "emerald-night",
      labelKey: "settings.emeraldNight",
      colorScheme: "dark",
    });
    expect(getThemeColorScheme("emerald-night")).toBe("dark");
  });

  it("déclare Cobalt givré comme une palette claire", () => {
    expect(THEME_OPTIONS).toContainEqual({
      id: "cobalt-frost",
      labelKey: "settings.cobaltFrost",
      colorScheme: "light",
    });
    expect(getThemeColorScheme("cobalt-frost")).toBe("light");
  });

  it("déclare Brume astrale comme une palette sombre", () => {
    expect(THEME_OPTIONS).toContainEqual({
      id: "astral-mist",
      labelKey: "settings.astralMist",
      colorScheme: "dark",
    });
    expect(getThemeColorScheme("astral-mist")).toBe("dark");
  });

  it("déclare Éclipse écarlate comme une palette sombre", () => {
    expect(THEME_OPTIONS).toContainEqual({
      id: "crimson-eclipse",
      labelKey: "settings.crimsonEclipse",
      colorScheme: "dark",
    });
    expect(getThemeColorScheme("crimson-eclipse")).toBe("dark");
  });

  it("valide uniquement les choix de thème connus", () => {
    expect(isThemeChoice("emerald-night")).toBe(true);
    expect(isThemeChoice("cobalt-frost")).toBe(true);
    expect(isThemeChoice("astral-mist")).toBe(true);
    expect(isThemeChoice("crimson-eclipse")).toBe(true);
    expect(isThemeChoice("unknown-theme")).toBe(false);
    expect(isThemeChoice(null)).toBe(false);
  });

  it("résout le mode système selon la préférence du système", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
    expect(resolveTheme("emerald-night", false)).toBe("emerald-night");
    expect(resolveTheme("cobalt-frost", true)).toBe("cobalt-frost");
    expect(resolveTheme("astral-mist", false)).toBe("astral-mist");
    expect(resolveTheme("crimson-eclipse", false)).toBe("crimson-eclipse");
  });

  it("inclut les palettes personnalisées dans le cycle des thèmes", () => {
    expect(getNextThemeChoice("dark")).toBe("emerald-night");
    expect(getNextThemeChoice("emerald-night")).toBe("cobalt-frost");
    expect(getNextThemeChoice("cobalt-frost")).toBe("astral-mist");
    expect(getNextThemeChoice("astral-mist")).toBe("crimson-eclipse");
    expect(getNextThemeChoice("crimson-eclipse")).toBe("system");
    expect(getNextThemeChoice("system")).toBe("light");
  });
});
