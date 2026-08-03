import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";
import { RESOLVED_THEME_OPTIONS, type ResolvedTheme } from "@/lib/app-themes";

const indexHtml = readFileSync("index.html", "utf8");
const bootstrapSource = indexHtml.match(/<script>([\s\S]*?)<\/script>/)?.[1];

function runBootstrap(choice: string | null, prefersDark = false) {
  const attributes: Record<string, string> = {};
  const context = {
    localStorage: { getItem: () => choice },
    window: { matchMedia: () => ({ matches: prefersDark }) },
    document: {
      documentElement: {
        setAttribute: (name: string, value: string) => {
          attributes[name] = value;
        },
      },
    },
  };

  expect(bootstrapSource).toBeDefined();
  runInNewContext(bootstrapSource!, context);
  return attributes;
}

function themeTokens(theme: ResolvedTheme): { background: string; mark: string } {
  const themePath = `src/styles/themes/${theme}.css`;
  // Le chemin vient exclusivement de la liste interne et bornée des thèmes.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const css = readFileSync(themePath, "utf8");
  const background = css.match(/--void:\s*(#[0-9a-f]{6});/i)?.[1];
  const mark = css.match(/--ink:\s*(#[0-9a-f]{6});/i)?.[1];

  expect(background).toBeDefined();
  expect(mark).toBeDefined();
  return { background: background!, mark: mark! };
}

function splashRule(theme: ResolvedTheme): string {
  const selector = theme === "light" || theme === "dark"
    ? `[data-theme="${theme}"] #splash`
    : `[data-palette="${theme}"] #splash`;
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const rule = indexHtml.match(new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`))?.[1];

  expect(rule).toBeDefined();
  return rule!;
}

describe("thème du splash de démarrage", () => {
  it.each(RESOLVED_THEME_OPTIONS)(
    "applique immédiatement la palette $id",
    ({ id, colorScheme }) => {
      expect(runBootstrap(id)).toMatchObject({
        "data-theme": colorScheme,
        "data-palette": id,
      });

      const tokens = themeTokens(id);
      const rule = splashRule(id);
      expect(rule).toContain(`background: ${tokens.background};`);
      expect(rule).toContain(`--splash-mark: ${tokens.mark};`);
    },
  );

  it("résout le mode système et les valeurs inconnues sans flash incohérent", () => {
    expect(runBootstrap("system", false)).toMatchObject({
      "data-theme": "light",
      "data-palette": "light",
    });
    expect(runBootstrap("system", true)).toMatchObject({
      "data-theme": "dark",
      "data-palette": "dark",
    });
    expect(runBootstrap("unknown-theme", true)).toMatchObject({
      "data-theme": "dark",
      "data-palette": "dark",
    });
  });
});
