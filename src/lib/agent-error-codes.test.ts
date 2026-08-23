import { describe, expect, it } from "vitest";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itCatalog from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";
import { isKnownAgentErrorCode, KNOWN_ERROR_KEYS } from "./agent-error-codes";

const catalogs: ReadonlyArray<Record<string, unknown>> = [fr, en, es, de, itCatalog, zh, ja];

describe("KNOWN_ERROR_KEYS", () => {
  it("pointe vers un message traduit dans les sept langues", () => {
    for (const translationKey of Object.values(KNOWN_ERROR_KEYS)) {
      for (const catalog of catalogs) {
        expect(readTranslation(catalog, translationKey)).not.toBeUndefined();
      }
    }
  });

  it("expose le refus Fast avec le texte exact dans les sept langues", () => {
    expect(isKnownAgentErrorCode("service_tier_unavailable")).toBe(true);
    expect(
      catalogs.map((catalog) => readTranslation(catalog, "errors.serviceTierUnavailable")),
    ).toEqual([
      "Le mode Rapide n'est pas disponible pour cette requête. Désactive-le ou choisis un modèle compatible.",
      "Fast mode is not available for this request. Turn it off or choose a compatible model.",
      "El modo Rápido no está disponible para esta solicitud. Desactívalo o elige un modelo compatible.",
      "Der Schnellmodus ist für diese Anfrage nicht verfügbar. Deaktiviere ihn oder wähle ein kompatibles Modell.",
      "La modalità Rapida non è disponibile per questa richiesta. Disattivala o scegli un modello compatibile.",
      "快速模式不适用于此请求。请将其关闭或选择兼容的模型。",
      "高速モードはこのリクエストでは利用できません。無効にするか、対応モデルを選択してください。",
    ]);
  });
});

function readTranslation(catalog: Record<string, unknown>, path: string): unknown {
  return path.split(".").reduce<unknown>((value, segment) => {
    if (!value || typeof value !== "object") return undefined;
    return (value as Record<string, unknown>)[segment];
  }, catalog);
}
