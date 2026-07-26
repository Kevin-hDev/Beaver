import { describe, it, expect, vi } from "vitest";
import { formatDateTime, formatRunStatus, formatSchedule } from "@/lib/wakeup-format";

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key, language: "fr" },
}));

describe("formatSchedule", () => {
  describe("once", () => {
    it("formate une date valide avec jour, mois court et heure", () => {
      const result = formatSchedule({ kind: "once", datetime: "2026-05-09T14:30" });
      expect(result).toBe("9 mai · 14h30");
    });

    it("retourne le string brut si le datetime est invalide", () => {
      const result = formatSchedule({ kind: "once", datetime: "date-invalide" });
      expect(result).toBe("date-invalide");
    });
  });

  describe("daily", () => {
    it("formate une heure valide avec le label daily", () => {
      const result = formatSchedule({ kind: "daily", time: "09:30" });
      expect(result).toBe("09h30 · wakeupFormat.daily");
    });

    it("retourne le string brut si le time est invalide", () => {
      const result = formatSchedule({ kind: "daily", time: "heure-invalide" });
      expect(result).toBe("heure-invalide");
    });
  });

  describe("weekly", () => {
    it("formate le jour court, l'heure et le label weekly", () => {
      // weekday 0 = lundi (new Date(2000, 0, 3) = lundi)
      const result = formatSchedule({ kind: "weekly", weekday: 0, time: "08:00" });
      expect(result).toBe("lun. 08h00 · wakeupFormat.weekly");
    });

    it("retourne uniquement le jour si le time est invalide", () => {
      const result = formatSchedule({ kind: "weekly", weekday: 0, time: "heure-invalide" });
      expect(result).toBe("lun.");
    });
  });

  describe("status", () => {
    it("formate un statut de run", () => {
      expect(formatRunStatus("missed")).toBe("heartbeat.status.missed");
      expect(formatRunStatus(null)).toBe("heartbeat.status.never");
    });

    it("formate none pour une date absente ou invalide", () => {
      expect(formatDateTime(null)).toBe("heartbeat.status.none");
      expect(formatDateTime("invalid")).toBe("heartbeat.status.none");
    });
  });
});
