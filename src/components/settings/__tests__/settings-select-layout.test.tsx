import { readFileSync } from "node:fs";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SettingsSelect } from "../settings-select";

/* Disposition d'une ligne : l'icône et le libellé à gauche, la coche au bord
   droit. Elle était à gauche, devant le libellé, et poussait tout le texte vers
   l'intérieur. Le panneau, lui, s'aligne sur le bord gauche du bouton : aligné
   à droite, il partait vers la gauche et recouvrait ce qui précède le bouton. */

const OPTIONS = [
  { value: "a", label: "Tous les projets" },
  { value: "b", label: "Discussions" },
];

function openList() {
  render(<SettingsSelect options={OPTIONS} value="a" onChange={() => {}} />);
  fireEvent.click(screen.getByText("Tous les projets", { selector: ".ss-trigger-label" }));
}

describe("disposition d'une ligne de SettingsSelect", () => {
  it("place la coche en dernier, après le libellé", () => {
    openList();
    const row = screen.getByText("Tous les projets", { selector: ".menu-row-label" }).closest(".ss-option");
    expect(row?.lastElementChild?.className).toContain("ss-option-check");
  });

  it("garde la place de la coche sur les lignes non cochées", () => {
    openList();
    const row = screen.getByText("Discussions", { selector: ".menu-row-label" }).closest(".ss-option");
    const check = row?.lastElementChild;
    expect(check?.className).toContain("ss-option-check");
    expect(check?.children).toHaveLength(0);
  });

  it("aligne le panneau sur le bord gauche du bouton", () => {
    const css = readFileSync("src/components/settings/settings-select.css", "utf8");
    const panel = css.slice(css.indexOf("\n.ss-panel {"), css.indexOf("}", css.indexOf("\n.ss-panel {")));
    expect(panel).toContain("left: 0;");
    expect(panel).not.toContain("right: 0;");
  });
});
