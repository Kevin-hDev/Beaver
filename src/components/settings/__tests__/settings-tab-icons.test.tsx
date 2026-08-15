import { readFileSync } from "node:fs";
import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SETTINGS_SECTIONS } from "../settings-sections";
import { ArchivedChatsIcon } from "../settings-tab-icons";
import { ArchiveBoxIcon } from "@/components/ui/archive-box-icon";

const TOKENS = readFileSync("src/styles/tokens-icon-sizes.css", "utf8");
const TABS = SETTINGS_SECTIONS.flatMap((section) => section.tabs);

function drawing(node: React.ReactElement): SVGSVGElement {
  const { container } = render(node);
  const svg = container.querySelector("svg");
  if (!svg) throw new Error("aucun dessin rendu");
  return svg;
}

describe("dessins des onglets de Réglages", () => {
  it("donne un dessin distinct à chaque onglet", () => {
    const icons = TABS.map((tab) => tab.icon);

    expect(icons.every((icon) => typeof icon === "function")).toBe(true);
    expect(new Set(icons).size).toBe(icons.length);
  });

  /* Le cadrage sert d'échelle au tracé : sans lui le navigateur rend le dessin
     à sa taille intrinsèque, et il déborde de la ligne au lieu de s'y ajuster. */
  it("cadre chaque dessin et le pose à la taille de la colonne", () => {
    for (const tab of TABS) {
      const svg = drawing(<tab.icon />);

      expect(svg.getAttribute("viewBox")).toMatch(/^0 0 \d+ \d+$/);
      expect(svg.style.width).toBe("var(--nav-icon-size)");
      expect(svg.style.height).toBe("var(--nav-icon-size)");
    }
  });

  /* Un tracé dont la couleur est écrite en dur reste identique d'un thème à
     l'autre et ne se distingue plus une fois l'onglet sélectionné. */
  it("laisse chaque tracé hériter de la couleur de l'onglet", () => {
    for (const tab of TABS) {
      for (const shape of drawing(<tab.icon />).querySelectorAll("*")) {
        for (const attribute of ["fill", "stroke"]) {
          const value = shape.getAttribute(attribute);
          if (value !== null) expect([attribute, value]).toEqual([attribute, expect.stringMatching(/^(currentColor|none)$/)]);
        }
      }
    }
  });

  /* Archiver une session et consulter les conversations archivées mènent au
     même endroit : deux dessins qui divergeraient laisseraient croire à deux
     destinations distinctes. */
  it("partage son dessin d'archive avec l'action qui archive une session", () => {
    expect(drawing(<ArchivedChatsIcon />).innerHTML).toBe(drawing(<ArchiveBoxIcon />).innerHTML);
  });

  it("réduit les deux tailles d'un cinquième sous l'icône de référence", () => {
    expect(TOKENS).toContain("--nav-icon-size: calc(var(--icon-md) * 0.8);");
    expect(TOKENS).toContain("--session-icon-size: calc(var(--icon-sm) * 0.8);");
    expect(TOKENS).toContain("--project-folder-icon-size: calc(var(--session-icon-size) * 1.2);");
  });
});
