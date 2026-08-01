import type { TFunction } from "i18next";
import { describe, expect, it } from "vitest";
import { toolErrorMessage } from "./tool-error-message";

const translations: Record<string, string> = {
  "agentLocal.toolActivity.errorCategories.conflict": "L’état actuel empêche cette opération.",
  "errors.toolFailed": "L’outil a échoué",
  "extensions.errors.codes.extensions_host_unavailable": "L’hôte d’extensions est indisponible.",
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

  it("réutilise la traduction précise d'une extension connue", () => {
    expect(toolErrorMessage("extension", "host failed", {
      code: "extensions_host_unavailable",
      category: "unavailable",
      retryable: true,
    }, t)).toBe("L’hôte d’extensions est indisponible.");
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
