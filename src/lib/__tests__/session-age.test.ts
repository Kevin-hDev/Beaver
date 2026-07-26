import { describe, expect, it } from "vitest";
import { getSessionAge } from "../session-age";

const now = Date.UTC(2026, 5, 30, 12, 0, 0);

function isoBefore(ms: number): string {
  return new Date(now - ms).toISOString();
}

describe("getSessionAge", () => {
  it("retourne les minutes avec un minimum de 1", () => {
    expect(getSessionAge(isoBefore(10_000), now)).toEqual({ count: 1, unit: "minute" });
    expect(getSessionAge(isoBefore(5 * 60_000), now)).toEqual({ count: 5, unit: "minute" });
  });

  it("retourne les heures", () => {
    expect(getSessionAge(isoBefore(2 * 60 * 60_000), now)).toEqual({ count: 2, unit: "hour" });
  });

  it("retourne les jours", () => {
    expect(getSessionAge(isoBefore(3 * 24 * 60 * 60_000), now)).toEqual({ count: 3, unit: "day" });
  });

  it("retourne les mois sur une base de 30 jours", () => {
    expect(getSessionAge(isoBefore(4 * 30 * 24 * 60 * 60_000), now)).toEqual({ count: 4, unit: "month" });
  });

  it("retourne les années sur une base de 365 jours", () => {
    expect(getSessionAge(isoBefore(2 * 365 * 24 * 60 * 60_000), now)).toEqual({ count: 2, unit: "year" });
  });

  it("ignore les dates invalides et limite les dates futures à 1 minute", () => {
    expect(getSessionAge("not-a-date", now)).toBeNull();
    expect(getSessionAge(new Date(now + 60_000).toISOString(), now)).toEqual({ count: 1, unit: "minute" });
  });
});
