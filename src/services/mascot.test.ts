import { describe, expect, it } from "vitest";
import {
  MASCOT_SIZE_MAX,
  normalizeMascotSettings,
  normalizeMascotState,
} from "./mascot";

describe("mascot service validation", () => {
  it("borne la taille et refuse une mascotte inconnue", () => {
    const settings = normalizeMascotSettings({
      enabled: true,
      mascot_id: "unknown",
      size_percent: 900,
    });

    expect(settings.enabled).toBe(true);
    expect(settings.mascot_id).toBe("cl-go-beaver");
    expect(settings.size_percent).toBe(MASCOT_SIZE_MAX);
  });

  it("rejette une position hors limites", () => {
    const settings = normalizeMascotSettings({
      position: { x: 100_001, y: 20 },
    });

    expect(settings.position).toBeNull();
  });

  it("accepte Circuit comme mascotte disponible", () => {
    const settings = normalizeMascotSettings({
      mascot_id: "circuit",
      size_percent: 100,
    });

    expect(settings.mascot_id).toBe("circuit");
  });

  it("accepte Kova comme mascotte disponible", () => {
    const settings = normalizeMascotSettings({
      mascot_id: "kova",
      size_percent: 100,
    });

    expect(settings.mascot_id).toBe("kova");
  });

  it("accepte Nival comme mascotte disponible", () => {
    const settings = normalizeMascotSettings({
      mascot_id: "nival",
      size_percent: 100,
    });

    expect(settings.mascot_id).toBe("nival");
  });

  it("retombe sur idle pour un état temps réel invalide", () => {
    expect(normalizeMascotState({ animation: "unknown", revision: -1 })).toEqual({
      animation: "idle",
      revision: 0,
    });
  });
});
