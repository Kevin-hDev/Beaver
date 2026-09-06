import { render, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SettingsSelect } from "../settings-select";

/* Disposition d'une ligne : l'icône et le libellé à gauche, la coche au bord
   droit. Elle était à gauche, devant le libellé, et poussait tout le texte vers
   l'intérieur. Le panneau, lui, sort de la page par un portail — sans quoi le
   fondu sous le titre figé lui retire son flou et la carte le coupe — et reste
   aligné sur le bord gauche du bouton : aligné à droite, il partait vers la
   gauche et recouvrait ce qui précède le bouton. */

const { floatingCalls } = vi.hoisted(() => ({ floatingCalls: [] as unknown[][] }));

vi.mock("@/hooks/use-floating-menu-position", async (importOriginal) => {
  const actual = await importOriginal<typeof import("@/hooks/use-floating-menu-position")>();
  return {
    ...actual,
    useFloatingMenuPosition: (...args: Parameters<typeof actual.useFloatingMenuPosition>) => {
      floatingCalls.push(args);
      return actual.useFloatingMenuPosition(...args);
    },
  };
});

const OPTIONS = [
  { value: "a", label: "Tous les projets" },
  { value: "b", label: "Discussions" },
];

function openList() {
  const rendered = render(<SettingsSelect options={OPTIONS} value="a" onChange={() => {}} />);
  fireEvent.click(screen.getByText("Tous les projets", { selector: ".ss-trigger-label" }));
  return rendered;
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

  it("porte le panneau hors du conteneur du composant", () => {
    const { container } = openList();
    expect(container.querySelector(".ss-panel")).toBeNull();
    expect(document.body.querySelector(".ss-panel")).not.toBeNull();
  });

  it("aligne le panneau sur le bord gauche du bouton", () => {
    floatingCalls.length = 0;
    openList();
    expect(floatingCalls.length).toBeGreaterThan(0);
    expect(floatingCalls.every((args) => args[1] === "left")).toBe(true);
  });
});
