/* @vitest-environment jsdom */
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { NAV_ITEMS } from "../nav-items";

afterEach(() => {
  cleanup();
});

function drawing(node: React.ReactElement): SVGSVGElement {
  const { container } = render(node);
  const svg = container.querySelector("svg");
  if (!svg) throw new Error("aucun dessin rendu");
  return svg;
}

describe("dessins de la rangée de navigation", () => {
  /* La rangée ne montre que le dessin : son libellé vit dans l'infobulle. Deux
     entrées au même signe deviendraient impossibles à départager. */
  it("donne un dessin distinct à chaque section", () => {
    const marks = NAV_ITEMS.map((item) => drawing(<item.icon />).innerHTML);

    expect(new Set(marks).size).toBe(NAV_ITEMS.length);
  });

  it("pose chaque dessin à la taille des entrées de navigation", () => {
    for (const item of NAV_ITEMS) {
      const svg = drawing(<item.icon />);

      expect(svg.getAttribute("viewBox")).toBe("0 0 24 24");
      expect(svg.style.width).toBe("var(--nav-icon-size)");
      expect(svg.style.height).toBe("var(--nav-icon-size)");
    }
  });

  /* Le bouton porte déjà le nom de la section : lu une seconde fois, le dessin
     ferait annoncer l'entrée deux fois de suite. */
  it("laisse le nom de la section au bouton qui les porte", () => {
    for (const item of NAV_ITEMS) {
      expect(drawing(<item.icon />).getAttribute("aria-hidden")).toBe("true");
    }
  });
});
