import type { TFunction } from "i18next";
import { describe, expect, it } from "vitest";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itCatalog from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";
import {
  ADMISSION_ERROR_CODES,
  admissionErrorKey,
  admissionErrorMessage,
} from "./admission-error";

const expected = [
  ["app-shutting-down", "errors.admission.appShuttingDown"],
  ["app-work-capacity-reached", "errors.admission.appCapacity"],
  ["service-shutting-down", "errors.admission.serviceShuttingDown"],
  ["service-work-capacity-reached", "errors.admission.serviceCapacity"],
  ["gateway-shutting-down", "errors.admission.gatewayShuttingDown"],
  ["gateway-busy", "errors.admission.gatewayBusy"],
  ["active-stream-limit-reached", "errors.admission.activeStreamCapacity"],
  ["stream-replaced", "errors.admission.streamReplaced"],
  ["subagent-read-only", "errors.admission.subagentReadOnly"],
] as const;
const expectedTranslationKeys = [
  "activeStreamCapacity",
  "appCapacity",
  "appShuttingDown",
  "gatewayBusy",
  "gatewayShuttingDown",
  "serviceCapacity",
  "serviceShuttingDown",
  "streamReplaced",
  "subagentReadOnly",
];
const catalogs = [
  ["en", en], ["fr", fr], ["es", es], ["de", de],
  ["it", itCatalog], ["zh", zh], ["ja", ja],
] as const;

const t = ((key: string) => key) as TFunction;

describe("admission-error", () => {
  it.each(expected)("mappe le code fermé %s", (code, key) => {
    expect(admissionErrorKey(code)).toBe(key);
    expect(admissionErrorKey(JSON.stringify(code))).toBe(key);
  });

  it("masque intégralement une erreur inconnue", () => {
    expect(admissionErrorKey("/private/session.json")).toBeNull();
    expect(admissionErrorMessage("/private/session.json", t)).toBe("errors.operationFailed");
    expect(admissionErrorKey("session-unavailable")).toBeNull();
    expect(admissionErrorMessage("session-unavailable", t)).toBe("errors.operationFailed");
  });

  it("expose exactement les neuf codes publics", () => {
    expect(ADMISSION_ERROR_CODES).toEqual(expected.map(([code]) => code));
  });

  it.each(catalogs)("traduit les neuf codes en %s", (_language, catalog) => {
    const admission = catalog.errors.admission;
    expect(Object.keys(admission).sort()).toEqual(expectedTranslationKeys);
    expect(Object.values(admission)).toHaveLength(9);
    expect(Object.values(admission).every((value) => value.trim().length > 0)).toBe(true);
  });
});
