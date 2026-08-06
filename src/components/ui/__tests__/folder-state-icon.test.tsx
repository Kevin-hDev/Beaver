import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { FolderStateIcon } from "../folder-state-icon";

function tracing(open: boolean): { path: string; svg: SVGSVGElement } {
  const { container } = render(<FolderStateIcon open={open} />);
  const svg = container.querySelector("svg");
  const path = svg?.querySelector("path")?.getAttribute("d");
  if (!svg || !path) throw new Error("aucun dessin rendu");
  return { path, svg };
}

describe("dessin de l'état d'un dossier de projet", () => {
  /* Les deux états sont le seul signal que la ligne donne sur le contenu du
     projet : rendus identiques, le repli deviendrait invisible. */
  it("change de tracé selon que le projet est déplié ou non", () => {
    expect(tracing(true).path).not.toBe(tracing(false).path);
  });

  /* Le dossier est posé plus grand que les autres dessins de sa ligne : son
     tracé occupe moins de son cadre, et à cadre égal il paraît plus petit. */
  it("se pose au-dessus des autres dessins de sa ligne", () => {
    for (const open of [true, false]) {
      const { svg } = tracing(open);

      expect(svg.style.width).toBe("var(--project-folder-icon-size)");
      expect(svg.style.height).toBe("var(--project-folder-icon-size)");
    }
  });

  /* Le nom du projet, juste à côté, porte déjà l'information : lu une seconde
     fois, le dessin ferait répéter la ligne à voix haute. */
  it("reste hors de l'arbre d'accessibilité", () => {
    expect(tracing(true).svg.getAttribute("aria-hidden")).toBe("true");
  });
});
