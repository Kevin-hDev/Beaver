import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  BranchSelectorBranchItem,
  BranchSelectorCreateItem,
  BranchSelectorWorktreeItem,
} from "../branch-selector-items";

/* Ordre des trois zones d'une ligne : le nom à gauche, la poubelle qui paraît au
   survol, la coche au bord droit.

   C'était l'inverse — coche au milieu, poubelle au bord — si bien que le survol
   faisait surgir une action destructive à l'endroit le plus exposé de la ligne,
   pendant que la marque de la branche courante flottait au milieu du vide. */

const DELETE = <button type="button" className="bs-delete-btn">poubelle</button>;

const branch = { name: "main", is_current: true, dirty_count: 0 };

function row(container: HTMLElement) {
  const node = container.querySelector(".bs-item");
  if (!node) throw new Error("ligne introuvable");
  return node;
}

describe("disposition d'une ligne du sélecteur de branches", () => {
  it("range la coche en dernier, après la poubelle", () => {
    const { container } = render(
      <BranchSelectorBranchItem branch={branch} onSelect={() => {}} deleteControl={DELETE} />,
    );
    const children = [...row(container).children].map((c) => c.className);
    expect(children).toEqual([
      expect.stringContaining("bs-item-select"),
      expect.stringContaining("bs-delete-btn"),
      "bs-item-check",
    ]);
  });

  it("sort la coche du bouton de sélection", () => {
    const { container } = render(
      <BranchSelectorBranchItem branch={branch} onSelect={() => {}} deleteControl={DELETE} />,
    );
    expect(container.querySelector(".bs-item-select .bs-item-check")).toBeNull();
    expect(container.querySelector(".bs-item-check svg")).not.toBeNull();
  });

  it("tient la place de la coche sur un worktree, qui n'en porte pas", () => {
    const { container } = render(
      <BranchSelectorWorktreeItem
        worktree={{ path: "/tmp/wt", branch: "feature" }}
        onSelect={() => {}}
        deleteControl={DELETE}
      />,
    );
    const check = row(container).lastElementChild;
    expect(check?.className).toBe("bs-item-check");
    expect(check?.children).toHaveLength(0);
  });

  it("coupe le libellé de création au lieu de le passer à la ligne", () => {
    const { container } = render(
      <BranchSelectorCreateItem label="Créer et extraire une nouvelle branche..." onStart={() => {}} />,
    );
    expect(container.querySelector(".menu-row-label")?.textContent)
      .toBe("Créer et extraire une nouvelle branche...");
  });
});
