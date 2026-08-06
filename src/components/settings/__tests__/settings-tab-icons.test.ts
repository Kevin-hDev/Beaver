import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { SETTINGS_SECTIONS } from "../settings-sections";

const ICONS = readFileSync("src/components/settings/settings-tab-icons.tsx", "utf8");

describe("dessins des onglets de Réglages", () => {
  it("donne un dessin à chaque onglet", () => {
    const withoutIcon = SETTINGS_SECTIONS
      .flatMap((section) => section.tabs)
      .filter((tab) => typeof tab.icon !== "function")
      .map((tab) => tab.id);

    expect(withoutIcon).toEqual([]);
  });

  it("ne réutilise pas le même dessin sur deux onglets", () => {
    const icons = SETTINGS_SECTIONS.flatMap((section) => section.tabs).map((tab) => tab.icon);

    expect(new Set(icons).size).toBe(icons.length);
  });

  /* Un dessin dont la couleur est écrite dans le tracé reste identique d'un
     thème à l'autre et ne se distingue plus une fois l'onglet sélectionné : il
     doit hériter de la couleur du texte qui l'accompagne. */
  it("laisse chaque tracé hériter de la couleur de l'onglet", () => {
    const painted = [...ICONS.matchAll(/(?:fill|stroke)="([^"]*)"/g)].map((match) => match[1]);
    const hardcoded = painted.filter((value) => value !== "currentColor" && value !== "none");

    expect(painted.length).toBeGreaterThan(0);
    expect(hardcoded).toEqual([]);
  });

  /* Le cadrage sert d'échelle au tracé : sans lui, le navigateur rend le dessin
     à sa taille intrinsèque et il déborde de la ligne au lieu de s'y ajuster. */
  it("cadre chaque dessin", () => {
    const declared = [...ICONS.matchAll(/viewBox="([^"]+)"/g)].length;
    const components = [...ICONS.matchAll(/^export function \w+Icon\(/gm)].length;

    expect(components).toBe(SETTINGS_SECTIONS.flatMap((section) => section.tabs).length);
    expect(declared).toBe(components);
  });
});
