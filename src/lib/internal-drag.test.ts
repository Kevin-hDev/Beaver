import { describe, it, expect, beforeEach } from "vitest";
import { setInternalDrag, isInternalDrag } from "./internal-drag";

describe("internal-drag", () => {
  // L'état du module est global → on le reset avant chaque test pour éviter
  // les fuites entre tests.
  beforeEach(() => {
    setInternalDrag(false);
  });

  it("retourne false par défaut", () => {
    expect(isInternalDrag()).toBe(false);
  });

  it("retourne true après setInternalDrag(true)", () => {
    setInternalDrag(true);
    expect(isInternalDrag()).toBe(true);
  });

  it("retourne false après setInternalDrag(false)", () => {
    setInternalDrag(true);
    setInternalDrag(false);
    expect(isInternalDrag()).toBe(false);
  });

  it("permet de basculer plusieurs fois", () => {
    setInternalDrag(true);
    expect(isInternalDrag()).toBe(true);
    setInternalDrag(false);
    expect(isInternalDrag()).toBe(false);
    setInternalDrag(true);
    expect(isInternalDrag()).toBe(true);
  });
});
