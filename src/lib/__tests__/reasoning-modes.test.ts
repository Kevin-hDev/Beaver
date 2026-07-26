import { describe, expect, it } from "vitest";
import {
  normalizeReasoningMode,
  reasoningModeOptions,
  type ReasoningMode,
} from "@/lib/reasoning-modes";
import type { AvailableModel } from "@/hooks/use-available-models";

function model(provider: string, id: string, reasoningModes?: ReasoningMode[]): AvailableModel {
  return {
    id,
    provider_id: provider,
    provider_name: provider,
    is_local: provider === "ollama",
    supports_tools: false,
    supports_thinking: true,
    reasoning_modes: reasoningModes,
  };
}

function modes(provider: string, id: string): ReasoningMode[] {
  return reasoningModeOptions(model(provider, id)).map((option) => option.mode);
}

describe("reasoning modes", () => {
  it("n'affiche pas OFF pour Codex OAuth", () => {
    expect(modes("codex-oauth", "gpt-5.5")).toEqual(["low", "medium", "high", "xhigh"]);
  });

  it("expose Max et Ultra uniquement aux modèles Codex compatibles", () => {
    expect(modes("codex-oauth", "gpt-5.6-sol")).toEqual([
      "low",
      "medium",
      "high",
      "xhigh",
      "max",
      "ultra",
    ]);
    expect(modes("codex-oauth", "gpt-5.6-terra")).toEqual([
      "low",
      "medium",
      "high",
      "xhigh",
      "max",
      "ultra",
    ]);
    expect(modes("codex-oauth", "gpt-5.6-luna")).toEqual([
      "low",
      "medium",
      "high",
      "xhigh",
      "max",
    ]);
  });

  it("affiche les niveaux Ollama GPT-OSS sans OFF", () => {
    expect(modes("ollama", "gpt-oss:20b")).toEqual(["low", "medium", "high"]);
  });

  it("adapte exactement les niveaux xAI actifs", () => {
    expect(modes("xai", "grok-4.5")).toEqual(["low", "medium", "high"]);
    expect(modes("xai", "grok-4.3")).toEqual(["off", "low", "medium", "high"]);
    expect(modes("xai", "grok-4.20-0309-reasoning")).toEqual(["auto"]);
    expect(modes("xai", "grok-build-0.1")).toEqual(["auto"]);
  });

  it("expose les niveaux GPT-5.6 pour OpenAI et OpenRouter", () => {
    const expected: ReasoningMode[] = ["off", "low", "medium", "high", "xhigh", "max"];
    expect(modes("openai", "gpt-5.6-sol")).toEqual(expected);
    expect(modes("openrouter", "openai/gpt-5.6-terra")).toEqual(expected);
    expect(modes("openrouter", "x-ai/grok-4.5")).toEqual(["low", "medium", "high"]);
  });

  it("adapte Groq selon la famille reasoning", () => {
    expect(modes("groq", "openai/gpt-oss-20b")).toEqual(["low", "medium", "high"]);
    expect(modes("groq", "qwen/qwen3-32b")).toEqual(["off", "auto"]);
    expect(modes("groq", "openai/gpt-oss-safeguard-20b")).toEqual(["auto"]);
  });

  it("adapte DeepSeek, Mistral et Moonshot", () => {
    expect(modes("deepseek", "deepseek-v4-pro")).toEqual(["off", "high", "xhigh"]);
    expect(modes("mistral", "mistral-small-latest")).toEqual(["off", "high"]);
    expect(modes("mistral", "magistral-medium-latest")).toEqual(["auto"]);
    expect(modes("mistral", "mistral-small-2506")).toEqual([]);
    expect(modes("moonshot", "kimi-k2.5")).toEqual(["off", "auto"]);
    expect(modes("moonshot", "kimi-k2-thinking")).toEqual(["auto"]);
    expect(modes("moonshot", "kimi-k2.7-code")).toEqual(["auto"]);
    expect(modes("moonshot", "k3")).toEqual(["low", "high", "max"]);
    expect(modes("moonshot-oauth", "k3")).toEqual(["low", "high", "max"]);
  });

  it("affiche OFF/AUTO pour Z.ai quand le modèle supporte le thinking", () => {
    expect(modes("zai", "glm-5")).toEqual(["off", "auto"]);
    expect(modes("zai", "glm-4.6")).toEqual(["off", "auto"]);
    expect(modes("zai", "glm-4.5-flash")).toEqual(["off", "auto"]);
  });

  it("adapte Gemini selon le modèle", () => {
    expect(modes("google", "gemini-3.5-flash")).toEqual(["low", "medium", "high"]);
    expect(modes("google", "gemini-2.5-pro")).toEqual(["low", "medium", "high"]);
    expect(modes("google", "gemini-2.5-flash")).toEqual(["off", "low", "medium", "high"]);
  });

  it("affiche les niveaux d'effort pour GLM-5.2", () => {
    expect(modes("zai", "glm-5.2")).toEqual(["off", "auto", "low", "medium", "high", "xhigh"]);
  });

  it("utilise les modes dynamiques exposés par OpenRouter", () => {
    const openrouter = model("openrouter", "z-ai/glm-5.1", ["off", "auto", "low", "medium", "high", "xhigh"]);
    expect(reasoningModeOptions(openrouter).map((option) => option.mode)).toEqual([
      "off",
      "auto",
      "low",
      "medium",
      "high",
      "xhigh",
    ]);
  });

  it("normalise un mode invalide vers la valeur sûre", () => {
    const options = reasoningModeOptions(model("codex-oauth", "gpt-5.5"));
    expect(normalizeReasoningMode("off", options)).toBe("medium");
  });

  it("active le thinking en AUTO par défaut pour un modèle commutable", () => {
    const options = reasoningModeOptions(model("groq", "qwen/qwen3-32b"));
    expect(normalizeReasoningMode(null, options)).toBe("auto");
  });

  it("utilise le premier niveau actif quand MOYEN n'existe pas", () => {
    const options = reasoningModeOptions(model("deepseek", "deepseek-v4-pro"));
    expect(normalizeReasoningMode(null, options)).toBe("high");
  });

  it("conserve le choix explicite DÉSACTIVÉ de l'utilisateur", () => {
    const options = reasoningModeOptions(model("deepseek", "deepseek-v4-pro"));
    expect(normalizeReasoningMode("off", options)).toBe("off");
  });

  it("utilise Max par défaut pour K3", () => {
    const k3 = model("moonshot-oauth", "k3", ["low", "high", "max"]);
    k3.default_reasoning_mode = "max";
    const options = reasoningModeOptions(k3);

    expect(normalizeReasoningMode(null, options, k3.default_reasoning_mode)).toBe("max");
    expect(normalizeReasoningMode("off", options, k3.default_reasoning_mode)).toBe("max");
  });
});
