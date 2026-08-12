import type { TFunction } from "i18next";
import { describe, expect, it } from "vitest";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itCatalog from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";
import { WAKEUP_RUN_ERROR_CODES, wakeupRunErrorMessage } from "./wakeup-run-error";

const t = ((key: string) => key) as TFunction;

describe("wakeup-run-error", () => {
  const requiredKeys = [
    "failed",
    "rateLimited",
    "authenticationFailed",
    "ollamaUnavailable",
    "missedUnavailable",
    "schedulerStopping",
    "capacityReached",
  ];

  it("traduit un code stable et masque un ancien texte", () => {
    expect(wakeupRunErrorMessage({ error_code: "capacity_reached" }, t))
      .toBe("heartbeat.history.errors.capacityReached");
    expect(wakeupRunErrorMessage({ error: "/private/config.json" }, t))
      .toBe("heartbeat.history.errors.failed");
  });

  it.each([
    ["en", en], ["fr", fr], ["es", es], ["de", de],
    ["it", itCatalog], ["zh", zh], ["ja", ja],
  ])("traduit les erreurs de réveil en %s", (_language, catalog) => {
    const errors = catalog.heartbeat.history.errors;
    expect(Object.keys(errors)).toEqual(requiredKeys);
    expect(Object.values(errors)).toHaveLength(WAKEUP_RUN_ERROR_CODES.length);
    expect(Object.values(errors).every((value) => value.trim().length > 0)).toBe(true);
  });

  it("distingue en japonais une occurrence ratée d'un réveil jamais exécuté", () => {
    expect(ja.heartbeat.status.missed).not.toBe(ja.heartbeat.status.never);
  });
});
