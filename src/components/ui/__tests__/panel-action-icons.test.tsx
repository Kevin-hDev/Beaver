/* @vitest-environment jsdom */
import { readFileSync } from "node:fs";
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { CloseIcon, DocumentationIcon, FullscreenIcon, OpenExternalIcon } from "../panel-action-icons";

const TOKENS = readFileSync("src/styles/tokens-icon-sizes.css", "utf8");

afterEach(() => {
  cleanup();
});

function drawing(node: React.ReactElement): SVGSVGElement {
  const { container } = render(node);
  const svg = container.querySelector("svg");
  if (!svg) throw new Error("aucun dessin rendu");
  return svg;
}

const TOUS = [
  ["ouvrir ailleurs", <OpenExternalIcon key="o" />, "var(--icon-md)"],
  ["plein écran", <FullscreenIcon key="f" />, "var(--icon-md)"],
  ["fermer", <CloseIcon key="c" />, "var(--panel-close-icon-size)"],
  ["documentation", <DocumentationIcon key="d" />, "var(--chrome-icon-docs)"],
] as const;

describe("dessins des actions d'un panneau", () => {
  it("pose chacun à la taille de la rangée qui l'accueille", () => {
    for (const [nom, node, taille] of TOUS) {
      expect([nom, drawing(node).style.width]).toEqual([nom, taille]);
    }
  });

  /* Une croix de deux traits ne couvre que 42 % de son cadre, contre 67 à 100 %
     pour ses voisines : au format commun elle rapetissait dans sa rangée. */
  it("relève la croix au-dessus de l'icône de sa rangée", () => {
    expect(TOKENS).toContain("--panel-close-icon-size: calc(var(--icon-md) * 1.2);");
  });

  it("laisse chaque tracé hériter de la couleur du bouton", () => {
    for (const [nom, node] of TOUS) {
      for (const forme of drawing(node).querySelectorAll("path")) {
        for (const attribut of ["fill", "stroke"]) {
          const valeur = forme.getAttribute(attribut);
          if (valeur !== null) expect([nom, valeur]).toEqual([nom, expect.stringMatching(/^(currentColor|none)$/)]);
        }
      }
    }
  });

  /* Chaque bouton porte déjà son nom en infobulle ou en aria-label : le dessin
     lu une seconde fois ferait annoncer l'action deux fois. */
  it("restent hors de l'arbre d'accessibilité", () => {
    for (const [nom, node] of TOUS) {
      expect([nom, drawing(node).getAttribute("aria-hidden")]).toEqual([nom, "true"]);
    }
  });
});
