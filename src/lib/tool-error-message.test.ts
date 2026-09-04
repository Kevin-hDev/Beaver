import type { TFunction } from "i18next";
import { describe, expect, it } from "vitest";
import {
  toolErrorHasLocalizedMessage,
  toolErrorMessage,
  toolErrorResultIsMachineCode,
} from "./tool-error-message";

const translations: Record<string, string> = {
  "agentLocal.toolActivity.errorCategories.conflict": "L’état actuel empêche cette opération.",
  "agentLocal.toolActivity.errorCategories.unavailable": "L’outil est temporairement indisponible.",
  "agentLocal.toolActivity.webSearchRuntimeUnavailable": "La recherche locale est indisponible.",
  "extensions.errors.codes.extensions_listing_unavailable": "La liste des extensions est indisponible.",
  "extensions.errors.codes.extensions_inspection_invalid": "L’inspection des extensions est invalide.",
  "extensions.errors.codes.extensions_inspection_unavailable": "L’inspection des extensions est indisponible.",
  "errors.toolFailed": "L’outil a échoué",
};
const t = ((key: string) => translations[key] ?? key) as TFunction;

describe("toolErrorMessage", () => {
  it("traduit une erreur structurée sans exposer son code technique", () => {
    const message = toolErrorMessage("memory_edit", "stale", {
      code: "memory_edit_stale",
      category: "conflict",
      retryable: false,
    }, t);

    expect(message).toBe("L’état actuel empêche cette opération.");
    expect(message).not.toContain("memory_edit_stale");
  });

  it("traduit un vrai code d'outil d'extension par sa catégorie", () => {
    const message = toolErrorMessage("extension", "Extension indisponible.", {
      code: "extension_unavailable",
      category: "unavailable",
      retryable: true,
    }, t);

    expect(message).toBe("L’outil est temporairement indisponible.");
    expect(message).not.toContain("extension_unavailable");
  });

  it("traduit les trois nouveaux codes backend d'extension sans exposer leur code", () => {
    for (const [code, expected] of [
      ["extensions_listing_unavailable", "La liste des extensions est indisponible."],
      ["extensions_inspection_invalid", "L’inspection des extensions est invalide."],
      ["extensions_inspection_unavailable", "L’inspection des extensions est indisponible."],
    ]) {
      const message = toolErrorMessage("list_extensions", code, {
        code,
        category: "unavailable",
        retryable: false,
      }, t);

      expect(message).toBe(expected);
      expect(message).not.toContain(code);
    }
  });

  it("traduit le runtime SearXNG sans afficher le code backend", () => {
    const error = {
      code: "web_search_runtime_unavailable",
      category: "unavailable" as const,
      retryable: true,
    };

    expect(toolErrorMessage("web_search", "searxng_runtime_unavailable", error, t))
      .toBe("La recherche locale est indisponible.");
    expect(toolErrorHasLocalizedMessage(error)).toBe(true);
    expect(toolErrorResultIsMachineCode("searxng_runtime_unavailable")).toBe(true);
    expect(toolErrorResultIsMachineCode("SearXNG: runtime unavailable")).toBe(false);
  });

  it("se replie sur l'erreur réelle nettoyée sans métadonnée", () => {
    expect(toolErrorMessage(
      "custom_tool",
      "Request failed token=very-secret-token\ninternal detail",
      undefined,
      t,
    )).toBe("Request failed token=[redacted]");
  });

  it("utilise un message générique si aucun détail n'est disponible", () => {
    expect(toolErrorMessage("custom_tool", "", undefined, t)).toBe("L’outil a échoué");
  });
});
