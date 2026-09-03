import { describe, expect, it } from "vitest";
import { UI_THEME_TOKENS } from "@/types/extension-ui-contract.generated";
import {
  extensionThemeChoice,
  parseExtensionThemeTokens,
  parseThemeCssDeclarations,
  resolvePublicThemeColor,
  themeIdFromChoice,
} from "../theme-parser";

describe("extension theme parser", () => {
  it("accepte uniquement les identifiants canoniques et les couleurs hexadécimales", () => {
    const choice = extensionThemeChoice("com.example", "com.example.theme.night");
    expect(choice).toBe("extension:com.example:com.example.theme.night");
    expect(themeIdFromChoice(choice)).toBe("com.example.theme.night");
    expect(parseExtensionThemeTokens({ "--void": "#010203", "--ink": "#AABBCCDD" }))
      .toEqual({ "--void": "#010203", "--ink": "#AABBCCDD" });

    expect(() => extensionThemeChoice("com.example", "../night")).toThrow("invalid_extension_theme");
    expect(() => parseExtensionThemeTokens({ "--void": "rgba(0, 0, 0, 1)" })).toThrow();
    expect(() => parseExtensionThemeTokens({ "--private-token": "#010203" })).toThrow();
    expect(() => parseExtensionThemeTokens(Object.fromEntries([
      ...UI_THEME_TOKENS.map((name) => [name, "#010203"]),
      ["--extra", "#010203"],
    ]))).toThrow();
  });

  it("résout les références bornées et refuse doublons, cycles et fonctions", () => {
    const declarations = parseThemeCssDeclarations(`
      :root {
        --void: var(--private-base);
        --private-base: #010203;
        --ink: rgba(255, 255, 255, 0.9);
      }
    `);
    expect(resolvePublicThemeColor("--void", declarations)).toBe("#010203");
    expect(resolvePublicThemeColor("--ink", declarations)).toBe("rgba(255, 255, 255, 0.9)");

    expect(() => parseThemeCssDeclarations(":root { --void: #000000; --void: #ffffff; }"))
      .toThrow();
    const cycle = parseThemeCssDeclarations(":root { --void: var(--ink); --ink: var(--void); }");
    expect(() => resolvePublicThemeColor("--void", cycle)).toThrow();
    for (const value of [
      "color-mix(in srgb, #000000, #ffffff)",
      "linear-gradient(#000000, #ffffff)",
      "var(--missing)",
      "rgba(256, 0, 0, 1)",
    ]) {
      const invalid = parseThemeCssDeclarations(`:root { --void: ${value}; }`);
      expect(() => resolvePublicThemeColor("--void", invalid)).toThrow();
    }
  });
});
