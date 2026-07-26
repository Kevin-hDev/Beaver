import { describe, expect, it } from "vitest";
import { dragDirectionAnimation, mascotPosition } from "./use-mascot-drag";

describe("déplacement de la mascotte", () => {
  it("choisit une pose selon la direction réelle", () => {
    expect(dragDirectionAnimation(8, 1)).toBe("move-right");
    expect(dragDirectionAnimation(-8, 1)).toBe("move-left");
    expect(dragDirectionAnimation(1, 8)).toBe("held");
    expect(dragDirectionAnimation(1, 1)).toBeNull();
  });

  it("garde le point attrapé sous le curseur", () => {
    expect(mascotPosition(250, 330, 20, 30)).toEqual({ x: 230, y: 300 });
  });

  it("refuse les coordonnées invalides ou excessives", () => {
    expect(mascotPosition(Number.NaN, 0, 0, 0)).toBeNull();
    expect(mascotPosition(2_000_000, 0, 0, 0)).toBeNull();
  });
});
