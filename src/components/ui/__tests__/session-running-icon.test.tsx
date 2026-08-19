import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import { SessionRunningIcon } from "../session-running-icon";

describe("SessionRunningIcon", () => {
  it("dessine une grille complète dont chaque carré porte sa propre boucle", () => {
    const { container } = render(<SessionRunningIcon />);
    const squares = container.querySelectorAll("path");
    expect(squares).toHaveLength(25);

    const loops = container.querySelectorAll("animateTransform");
    expect(loops).toHaveLength(25);
    /* Sans identifiant partagé : plusieurs sessions tournent en même temps dans
       la barre latérale, et un identifiant en double les lierait entre elles. */
    expect(container.querySelector("[id]")).toBeNull();

    loops.forEach((loop) => {
      const keyTimes = (loop.getAttribute("keyTimes") ?? "").split(";");
      const values = (loop.getAttribute("values") ?? "").split(";");
      /* Autant de repères que de valeurs, du début à la fin du cycle : SMIL
         ignore l'animation entière au premier écart. */
      expect(keyTimes).toHaveLength(values.length);
      expect(keyTimes[0]).toBe("0");
      expect(keyTimes[keyTimes.length - 1]).toBe("1");
      /* Elle part et revient à sa taille pleine, sinon la grille ne se referme
         pas en bloc plein entre deux tours. */
      expect(values[0]).toBe("1");
      expect(values[values.length - 1]).toBe("1");
      expect(loop.getAttribute("repeatCount")).toBe("indefinite");
    });
  });

  it("garde une seule autorité sur sa taille et sa couleur", () => {
    const { container } = render(<SessionRunningIcon />);
    const svg = container.querySelector("svg") as SVGElement;
    expect(svg.style.width).toBe("var(--session-running-icon-size)");
    expect(svg.getAttribute("aria-hidden")).toBe("true");
    expect(container.querySelector("[fill^='#']")).toBeNull();
    expect(container.querySelectorAll("[fill='currentColor']")).toHaveLength(25);
  });

  it("n'arrondit que les quatre coins extérieurs de la grille", () => {
    const { container } = render(<SessionRunningIcon />);
    const radii = Array.from(container.querySelectorAll("path")).map((cell) => {
      const arcs = Array.from((cell.getAttribute("d") ?? "").matchAll(/a([\d.]+) /g));
      /* Un arc de rayon zéro est rendu comme un trait droit, donc un coin
         carré : seuls les rayons non nuls comptent. */
      return arcs.filter((arc) => Number(arc[1]) > 0).length;
    });
    /* Quatre carrés arrondis, un coin chacun. Un cinquième signifierait une
       encoche au milieu d'un bord, là où deux carrés se touchent. */
    expect(radii.filter((count) => count > 0)).toEqual([1, 1, 1, 1]);
  });

  it("laisse les carrés se retirer chacun à leur tour, en un seul tour", () => {
    const { container } = render(<SessionRunningIcon />);
    /* L'instant du creux de chaque carré : c'est lui qui porte l'aiguille. */
    const troughs = Array.from(container.querySelectorAll("animateTransform")).map((loop) => {
      const keyTimes = (loop.getAttribute("keyTimes") ?? "").split(";").map(Number);
      const values = (loop.getAttribute("values") ?? "").split(";").map(Number);
      return keyTimes[values.indexOf(Math.min(...values))];
    });
    /* Le dernier carré se rallume avant la fin du cycle : c'est ce qui laisse la
       grille pleine un instant avant de repartir. */
    expect(Math.max(...troughs)).toBeLessThan(1);
    /* Cinq rayons par quart de tour au moins : sans étalement, les vingt-cinq
       carrés battraient ensemble et il n'y aurait plus d'aiguille. */
    expect(new Set(troughs).size).toBeGreaterThanOrEqual(16);
  });
});
