import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const reliefCss = readFileSync("src/styles/relief.css", "utf8");
const globalCss = readFileSync("src/styles/global.css", "utf8");
const tokensCss = readFileSync("src/styles/tokens.css", "utf8");
const chatCss = readFileSync("src/components/agent-local/chat.css", "utf8");
const chatInputTsx = readFileSync("src/components/agent-local/chat-input.tsx", "utf8");

const themeCss = {
  dark: readFileSync("src/styles/themes/dark.css", "utf8"),
  light: readFileSync("src/styles/themes/light.css", "utf8"),
  "emerald-night": readFileSync("src/styles/themes/emerald-night.css", "utf8"),
  "cobalt-frost": readFileSync("src/styles/themes/cobalt-frost.css", "utf8"),
  "astral-mist": readFileSync("src/styles/themes/astral-mist.css", "utf8"),
  "crimson-eclipse": readFileSync("src/styles/themes/crimson-eclipse.css", "utf8"),
};

const THEMES = Object.keys(themeCss) as (keyof typeof themeCss)[];

function token(css: string, name: string): string {
  const debut = css.indexOf(`${name}:`);
  if (debut === -1) throw new Error(`${name} absent`);
  const fin = css.indexOf(";", debut);
  return css.slice(debut + name.length + 1, fin).trim();
}

describe("Réglage de relief des six thèmes", () => {
  it.each(THEMES)("%s déclare les six valeurs vives", (theme) => {
    const css = themeCss[theme];
    for (const name of [
      "--relief-stroke-rgb",
      "--relief-stroke-top",
      "--relief-stroke-bottom",
      "--relief-shadow-y",
      "--relief-shadow-blur",
      "--relief-shadow-alpha",
    ]) {
      expect(() => token(css, name)).not.toThrow();
    }
  });

  it("prend de la lumière sur fond sombre et de l'encre sur fond clair", () => {
    for (const theme of ["dark", "emerald-night", "astral-mist", "crimson-eclipse"] as const) {
      expect(token(themeCss[theme], "--relief-stroke-rgb")).toBe("255, 255, 255");
    }
    for (const theme of ["light", "cobalt-frost"] as const) {
      expect(token(themeCss[theme], "--relief-stroke-rgb")).toBe("0, 0, 0");
    }
  });

  it("compense sur les deux fonds où l'ombre ne creuse plus", () => {
    const commun = Number(token(themeCss.dark, "--relief-stroke-top"));
    for (const theme of ["astral-mist", "crimson-eclipse"] as const) {
      expect(Number(token(themeCss[theme], "--relief-stroke-top"))).toBeGreaterThan(commun);
      expect(Number(token(themeCss[theme], "--relief-stroke-bottom"))).toBeGreaterThan(0);
      expect(token(themeCss[theme], "--relief-shadow-y")).toBe("5px");
    }
    expect(Number(token(themeCss.dark, "--relief-stroke-bottom"))).toBe(0);
    expect(Number(token(themeCss["emerald-night"], "--relief-stroke-bottom"))).toBe(0);
  });
});

describe("Primitive de relief", () => {
  it("est chargée par la feuille globale", () => {
    expect(globalCss).toContain('@import "./relief.css";');
  });

  it("dessine le trait dans une couronne masquée plutôt qu'une bordure", () => {
    expect(reliefCss).toContain("mask-composite: exclude");
    expect(reliefCss).toContain("-webkit-mask-composite: xor");
    expect(reliefCss).toMatch(/\.icon-btn-secondary\s*\{[^}]*border:\s*0;/s);
  });

  it("éteint le trait vers le bas", () => {
    expect(reliefCss).toContain("rgba(var(--relief-stroke-rgb), var(--relief-top))");
    expect(reliefCss).toContain("rgba(var(--relief-stroke-rgb), var(--relief-bottom))");
  });

  it("dérive les trois élévations du réglage du thème, sans valeur figée", () => {
    for (const name of ["--elev-rest", "--elev-float", "--elev-above"]) {
      const valeur = token(reliefCss, name);
      expect(valeur).toContain("var(--relief-shadow-y)");
      expect(valeur).toContain("var(--relief-shadow-blur)");
      expect(valeur).toContain("var(--relief-shadow-alpha)");
    }
  });

  it("allège le trait sur les écrans sans densité double", () => {
    expect(reliefCss).toContain("@media (max-resolution: 1.5dppx)");
  });

  it("creuse le champ au lieu de le poser", () => {
    const regle = reliefCss.match(/\.field\s*\{[^}]*\}/s)?.[0] ?? "";
    expect(regle).toContain("inset");
    expect(regle).not.toMatch(/box-shadow:\s*0 /);
  });

  it("ne dessine rien au focus clavier", () => {
    expect(reliefCss).toMatch(/\.relief:focus-visible,\s*\.relief:focus-within\s*\{\s*outline:\s*none;/s);
    expect(reliefCss).not.toMatch(/box-shadow[^;]*var\(--pulse/);
  });
});

describe("Arrondi unique", () => {
  it("n'a qu'une autorité, vers laquelle les anciens noms renvoient", () => {
    expect(tokensCss).toContain("--radius: 6px;");
    for (const name of ["--radius-xs", "--radius-sm", "--radius-md", "--radius-panel", "--radius-md-bubble"]) {
      expect(token(tokensCss, name)).toBe("var(--radius)");
    }
  });

  it("n'est plus déclaré une seconde fois dans la feuille globale", () => {
    expect(globalCss).not.toMatch(/@theme\s*\{[^}]*--radius/s);
  });
});

describe("Champ de saisie", () => {
  it("porte la primitive plutôt que son propre contour", () => {
    expect(chatInputTsx).toContain("chat-input-bubble relief elev-float");
  });

  it("n'a plus ni fond ni bordure propres", () => {
    const regle = chatCss.match(/\.chat-input-bubble\s*\{[^}]*\}/s)?.[0] ?? "";
    expect(regle).not.toContain("background:");
    expect(regle).not.toContain("border:");
    expect(regle).toContain("border-radius: var(--radius);");
  });
});
