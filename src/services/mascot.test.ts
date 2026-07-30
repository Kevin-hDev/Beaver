import { describe, expect, it } from "vitest";
import { MASCOT_IDS } from "@/types/mascot";
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

  it("accepte toutes les mascottes disponibles", () => {
    for (const mascotId of MASCOT_IDS) {
      const settings = normalizeMascotSettings({
        mascot_id: mascotId,
        size_percent: 100,
      });

      expect(settings.mascot_id).toBe(mascotId);
    }
  });

  it("retombe sur idle pour un état temps réel invalide", () => {
    expect(normalizeMascotState({ animation: "unknown", revision: -1 })).toEqual({
      animation: "idle",
      revision: 0,
    });
  });
});
