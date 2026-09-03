import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { RESOLVED_THEME_OPTIONS } from "@/lib/app-themes";
import {
  parseThemeCssDeclarations,
  resolvePublicThemeColor,
} from "@/features/extension-ui/themes/theme-parser";
import { UI_THEME_TOKENS } from "@/types/extension-ui-contract.generated";

describe("public theme contract", () => {
  it("dérive les six thèmes de l'autorité centrale et résout chaque jeton public", () => {
    expect(RESOLVED_THEME_OPTIONS).toHaveLength(6);
    expect(new Set(RESOLVED_THEME_OPTIONS.map(({ cssPath }) => cssPath)).size).toBe(6);

    for (const theme of RESOLVED_THEME_OPTIONS) {
      // Les chemins viennent uniquement du catalogue interne borné des thèmes Beaver.
      // eslint-disable-next-line security/detect-non-literal-fs-filename
      const css = readFileSync(theme.cssPath, "utf8");
      const declarations = parseThemeCssDeclarations(css);
      for (const token of UI_THEME_TOKENS) {
        expect(declarations.has(token), `${theme.id}: ${token}`).toBe(true);
        const value = resolvePublicThemeColor(token, declarations);
        expect(value.startsWith("#") || value.startsWith("rgba("), `${theme.id}: ${token}`)
          .toBe(true);
      }
    }
  });
});
