import { describe, expect, it } from "vitest";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";
import type { AvailableModel } from "@/hooks/use-available-models";

function model(
  modes: ReasoningMode[],
  overrides: Partial<AvailableModel> = {},
): AvailableModel {
  return {
    id: "modele-inconnu",
    provider_id: "provider-fictif",
    provider_name: "Provider fictif",
    is_local: false,
    supports_tools: false,
    supports_thinking: true,
    reasoning_modes: modes,
    context_usage_includes_reasoning: true,
    ...overrides,
  };
}

describe("reasoning modes", () => {
  it("utilise uniquement les modes fournis par les métadonnées du modèle", () => {
    expect(reasoningModeOptions(model(["low", "max"])).map((entry) => entry.mode))
      .toEqual(["low", "max"]);
  });

  it("ne présente pas le mode technique auto comme un niveau d'effort", () => {
    expect(reasoningModeOptions(model(
      ["off", "auto", "low", "high"],
      { provider_id: "anthropic" },
    )).map((entry) => entry.mode))
      .toEqual(["off", "low", "high"]);
  });

  it("ne modifie pas les modes auto des autres providers", () => {
    expect(reasoningModeOptions(model(["off", "auto", "high"])).map((entry) => entry.mode))
      .toEqual(["off", "auto", "high"]);
  });

  it("n’invente aucun mode quand la liste est absente ou vide", () => {
    expect(reasoningModeOptions(model([]))).toEqual([]);
    expect(reasoningModeOptions(model([], { reasoning_modes: undefined }))).toEqual([]);
  });

  it("masque les modes si le modèle ne prend pas le thinking en charge", () => {
    expect(reasoningModeOptions(model(["low"], { supports_thinking: false }))).toEqual([]);
  });

  it("normalise vers la valeur préférée fournie par Rust", () => {
    const options = reasoningModeOptions(model(["low", "high", "max"]));
    expect(normalizeReasoningMode("off", options, "max")).toBe("max");
  });

  it("conserve un choix explicite valide", () => {
    const options = reasoningModeOptions(model(["off", "auto", "high"]));
    expect(normalizeReasoningMode("off", options, "auto")).toBe("off");
  });

  it("garde les replis génériques sans connaître de provider", () => {
    expect(normalizeReasoningMode(null, reasoningModeOptions(model(["low", "medium", "high"]))))
      .toBe("medium");
    expect(normalizeReasoningMode(null, reasoningModeOptions(model(["off", "auto"]))))
      .toBe("auto");
    expect(normalizeReasoningMode(null, reasoningModeOptions(model(["off", "high"]))))
      .toBe("high");
  });
});
