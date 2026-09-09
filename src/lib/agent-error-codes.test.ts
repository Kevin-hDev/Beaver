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
  it.each(["provider_empty_response", "provider_output_limit", "provider_content_filtered"])(
    "distingue %s d’une erreur de clé dans les sept langues",
    (code) => {
      expect(isKnownAgentErrorCode(code)).toBe(true);
      expect(KNOWN_ERROR_KEYS[code]).not.toBe(KNOWN_ERROR_KEYS.auth_failed);
      for (const catalog of catalogs) {
        expect(readTranslation(catalog, KNOWN_ERROR_KEYS[code])).toEqual(expect.any(String));
      }
    },
  );
  it("reconnaît le prérequis de majorité sans le confondre avec une clé invalide", () => {
    expect(isKnownAgentErrorCode("provider_age_confirmation_required")).toBe(true);
    expect(KNOWN_ERROR_KEYS.provider_age_confirmation_required).not.toBe(KNOWN_ERROR_KEYS.auth_failed);
  });
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

  it("traduit l'indisponibilite du catalogue API dans les sept langues", () => {
    expect(isKnownAgentErrorCode("model_catalog_unavailable")).toBe(true);
    for (const catalog of catalogs) {
      expect(readTranslation(catalog, "errors.modelCatalogUnavailable")).not.toBeUndefined();
    }
  });

  it("distingue un réglage de raisonnement invalide d'un rejeu incompatible", () => {
    const key = KNOWN_ERROR_KEYS.reasoning_configuration_invalid;
    expect(key).toBe("errors.reasoningConfigurationInvalid");
    expect(key).not.toBe(KNOWN_ERROR_KEYS.reasoning_continuity_invalid);
    for (const catalog of catalogs) {
      expect(readTranslation(catalog, key)).toEqual(expect.any(String));
    }
    expect(readTranslation(fr, key)).toBe(
      "Le réglage de raisonnement n’est pas compatible avec ce modèle. Choisis un autre réglage puis réessaie.",
    );
  });

  it.each([
    ["session_inconsistent", "errors.sessionInconsistent"],
    ["model_invalid", "errors.modelInvalid"],
  ])("restaure le motif précis %s dans les sept langues", (code, key) => {
    expect(KNOWN_ERROR_KEYS[code]).toBe(key);
    for (const catalog of catalogs) {
      expect(readTranslation(catalog, key)).toEqual(expect.any(String));
    }
  });
});

function readTranslation(catalog: Record<string, unknown>, path: string): unknown {
  return path.split(".").reduce<unknown>((value, segment) => {
    if (!value || typeof value !== "object") return undefined;
    return (value as Record<string, unknown>)[segment];
  }, catalog);
}
