import { describe, expect, it } from "vitest";
import { legacyXaiReplacement } from "./model-migrations";

describe("legacyXaiReplacement", () => {
  it("normalise les anciens identifiants Grok 4.20", () => {
    expect(
      legacyXaiReplacement("xai", "grok-4.20-reasoning", "high", ["auto"]),
    ).toEqual({
      model: "grok-4.20-0309-reasoning",
      reasoningMode: "auto",
    });
    expect(
      legacyXaiReplacement("xai", "grok-4.20-non-reasoning", "off", []),
    ).toEqual({
      model: "grok-4.20-0309-non-reasoning",
      reasoningMode: null,
    });
  });

  it("conserve le choix utilisateur quand le nouveau modèle le supporte", () => {
    expect(
      legacyXaiReplacement("xai", "grok-4-fast-reasoning", "high", [
        "off",
        "low",
        "medium",
        "high",
      ]),
    ).toEqual({ model: "grok-4.3", reasoningMode: "high" });
  });

  it("applique le remplacement officiel quand le choix est incompatible", () => {
    expect(
      legacyXaiReplacement("xai", "grok-4-fast-reasoning", "auto", [
        "off",
        "low",
        "medium",
        "high",
      ]),
    ).toEqual({ model: "grok-4.3", reasoningMode: "low" });
    expect(
      legacyXaiReplacement("xai", "grok-4-fast-non-reasoning", "auto", [
        "off",
        "low",
        "medium",
        "high",
      ]),
    ).toEqual({ model: "grok-4.3", reasoningMode: "off" });
  });

  it("remplace les anciens modèles code par Grok Build", () => {
    expect(
      legacyXaiReplacement("xai", "grok-code-fast", "medium", ["auto"]),
    ).toEqual({ model: "grok-build-0.1", reasoningMode: "auto" });
  });

  it("ignore les autres providers et les modèles actuels", () => {
    expect(legacyXaiReplacement("openai", "grok-3", null, [])).toBeNull();
    expect(legacyXaiReplacement("xai", "grok-4.5", null, [])).toBeNull();
  });
});
