import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ArchiveBoxIcon } from "../archive-box-icon";
import { RenameIcon } from "../rename-icon";

function drawing(node: React.ReactElement): SVGSVGElement {
  const { container } = render(node);
  const svg = container.querySelector("svg");
  if (!svg) throw new Error("aucun dessin rendu");
  return svg;
}

describe("dessins des actions sur une session", () => {
  /* Ces dessins sont posés sans taille par les menus qui les portent : un
     défaut manquant les rendrait à leur taille intrinsèque, hors de la ligne. */
  it("se posent à la taille des menus de la barre latérale", () => {
    for (const node of [<ArchiveBoxIcon key="archive" />, <RenameIcon key="rename" />]) {
      const svg = drawing(node);

      expect(svg.style.width).toBe("var(--session-icon-size)");
      expect(svg.style.height).toBe("var(--session-icon-size)");
    }
  });

  /* Le dessin accompagne un libellé ou un bouton qui porte déjà son nom : lu
     une seconde fois, il ferait répéter l'action à voix haute. */
  it("restent hors de l'arbre d'accessibilité", () => {
    for (const node of [<ArchiveBoxIcon key="archive" />, <RenameIcon key="rename" />]) {
      expect(drawing(node).getAttribute("aria-hidden")).toBe("true");
    }
  });
});
