import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const cardCss = readFileSync("src/components/ui/browse-card.css", "utf8");

const THEMES = [
  "dark", "light", "emerald-night", "cobalt-frost",
  "astral-mist", "crimson-eclipse",
];

function themeCss(name: string): string {
  // Le nom vient de la liste ci-dessus, pas d'une entrée.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return readFileSync(`src/styles/themes/${name}.css`, "utf8");
}

function cardBg(css: string): string {
  return /--card-on-glass:\s*([^;]+);/.exec(css)?.[1] ?? "";
}

function chipBg(css: string): string {
  return /--chip-bg:\s*([^;]+);/.exec(css)?.[1] ?? "";
}

function alphaOf(value: string): number {
  const rgba = /rgba\([^)]*,\s*([\d.]+)\s*\)/.exec(value);
  // Une couleur sans canal alpha (#rrggbb, rgb(...)) est pleinement opaque.
  return rgba ? Number.parseFloat(rgba[1]) : 1;
}

describe("card de catalogue", () => {
  it("laisse le flou du panneau traverser la card", () => {
    // Ces cards sont posées sur un panneau en verre dépoli. Un fond opaque
    // masquerait le flou derrière chacune d'elles et l'effet disparaîtrait
    // là où il se voit le plus (demande du propriétaire, 2026-07-30).
    const opaque = THEMES.filter((name) => alphaOf(cardBg(themeCss(name))) >= 1);

    expect(opaque).toEqual([]);
  });

  it("détache la pastille du fond de la card", () => {
    // --chip-bg posait auparavant --shell, qui vaut --surface-glass en thème
    // sombre : la pastille avait la couleur exacte de la card et disparaissait.
    for (const name of THEMES) {
      const css = themeCss(name);

      expect(chipBg(css), `--chip-bg absent de ${name}`).not.toBe("");
      expect(chipBg(css), `--chip-bg identique à la card dans ${name}`)
        .not.toBe(cardBg(css));
    }
  });

  it("tire card et pastille des tokens de thème", () => {
    expect(cardCss).toContain("background: var(--card-on-glass);");
    expect(cardCss).toContain("background: var(--chip-bg);");
  });

  it("ne survole pas la card par son contour", () => {
    const hover = /\.browse-card:hover[^{]*\{([^}]*)\}/.exec(cardCss)?.[1] ?? "";

    expect(hover).toContain("background:");
    expect(hover).not.toContain("border-color:");
  });
});
