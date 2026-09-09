import { describe, expect, it } from "vitest";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";
import type { AvailableModel } from "@/hooks/use-available-models";
import type { ModelReasoningContract } from "@/types/model-reasoning-contract";

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

  it("consomme le contrôle normalisé et expose minimal sans autre effort inventé", () => {
    const reasoning_contract: ModelReasoningContract = {
      mandatory: true,
      control: { kind: "efforts", efforts: ["minimal", "high"] },
    };

    expect(reasoningModeOptions(model(["low"], { reasoning_contract })).map((entry) => entry.mode))
      .toEqual(["minimal", "high"]);
  });

  it("ne transforme pas un défaut provider natif en faux sélecteur", () => {
    const reasoning_contract: ModelReasoningContract = {
      mandatory: true,
      control: { kind: "provider_default" },
    };

    expect(reasoningModeOptions(model(["auto"], { reasoning_contract }))).toEqual([]);
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

  it("expose les modes des neuf couples et normalise un ancien off", () => {
    const contracts: Array<{
      provider_id: string;
      id: string;
      modes: ReasoningMode[];
      preferred: ReasoningMode;
    }> = [
      { provider_id: "google", id: "gemini-3.8-flash", modes: ["low", "medium", "high"], preferred: "medium" },
      { provider_id: "zai", id: "glm-5.3-flash", modes: ["low", "high", "max"], preferred: "max" },
      { provider_id: "openai", id: "gpt-6-astra", modes: ["low", "medium", "high", "xhigh", "max"], preferred: "medium" },
      { provider_id: "openrouter", id: "google/gemini-3.8-flash", modes: ["low", "medium", "high"], preferred: "medium" },
      { provider_id: "openrouter", id: "z-ai/glm-5.3-flash", modes: ["low", "high", "max"], preferred: "max" },
      { provider_id: "openrouter", id: "openai/gpt-6-astra", modes: ["low", "medium", "high", "xhigh", "max"], preferred: "medium" },
      // Artificial metadata: the exact Alibaba reasoning contract remains blocked.
      { provider_id: "qwen", id: "ZHIPU/GLM-5.3-Flash", modes: ["auto"], preferred: "auto" },
      { provider_id: "ollama", id: "glm-5.3-flash:cloud", modes: ["low", "high", "max"], preferred: "max" },
      { provider_id: "codex-oauth", id: "gpt-6-astra", modes: ["low", "medium", "high", "xhigh", "max"], preferred: "medium" },
    ];

    for (const contract of contracts) {
      const options = reasoningModeOptions(model(contract.modes, {
        provider_id: contract.provider_id,
        id: contract.id,
      }));
      expect(options.map((entry) => entry.mode)).toEqual(contract.modes);
      expect(normalizeReasoningMode("off", options, contract.preferred))
        .toBe(contract.preferred);
    }
  });
});
