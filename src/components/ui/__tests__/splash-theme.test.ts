import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";
import { RESOLVED_THEME_OPTIONS, type ResolvedTheme } from "@/lib/app-themes";

const indexHtml = readFileSync("index.html", "utf8");
const bootstrapSource = indexHtml.match(/<script>([\s\S]*?)<\/script>/)?.[1];

function runBootstrap(
  choice: string | null,
  prefersDark = false,
  userAgent = "Mozilla/5.0 (Macintosh)",
  base: string | null = null,
) {
  const attributes: Record<string, string> = {};
  const context = {
    localStorage: {
      getItem: (key: string) => key === "clgo-theme" ? choice : base,
    },
    navigator: { userAgent },
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
  const themePath = RESOLVED_THEME_OPTIONS.find(({ id }) => id === theme)?.cssPath;
  expect(themePath).toBeDefined();
  // Le chemin vient exclusivement de la liste interne et bornée des thèmes.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const css = readFileSync(themePath!, "utf8");
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
  const selectorStart = indexHtml.indexOf(selector);
  const bodyStart = indexHtml.indexOf("{", selectorStart);
  const bodyEnd = indexHtml.indexOf("}", bodyStart);
  const rule = selectorStart >= 0 && bodyStart >= 0 && bodyEnd > bodyStart
    ? indexHtml.slice(bodyStart + 1, bodyEnd)
    : undefined;

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

      /* Le castor a deux couleurs : la découpe posée dessous et l'encre par-dessus.
         Sur fond sombre la découpe prend --ink et l'encre --void ; sur fond clair
         les rôles s'échangent, la découpe valant alors la couleur du fond, donc
         invisible. Ces valeurs sont écrites en dur dans index.html parce que le
         splash peint avant les feuilles de thème : ce test est le seul lien qui
         les tient sur les jetons de leur palette. */
      const tokens = themeTokens(id);
      const rule = splashRule(id);
      const surfaceColor = colorScheme === "dark" ? tokens.mark : tokens.background;
      const markColor = colorScheme === "dark" ? tokens.background : tokens.mark;

      expect(rule).toContain(`background: ${tokens.background};`);
      expect(rule).toContain(`--splash-surface: ${surfaceColor};`);
      expect(rule).toContain(`--splash-mark: ${markColor};`);
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

  it("projette uniquement la base sûre d'un thème d'extension avant le catalogue", () => {
    expect(runBootstrap("extension:com.example.light", true, undefined, "light"))
      .toMatchObject({ "data-theme": "light", "data-palette": "light" });
    expect(runBootstrap("extension:com.example.dark", false, undefined, "dark"))
      .toMatchObject({ "data-theme": "dark", "data-palette": "dark" });
    expect(runBootstrap("extension:com.example.invalid", true, undefined, "corrupt"))
      .toMatchObject({ "data-theme": "dark", "data-palette": "dark" });
  });
});
