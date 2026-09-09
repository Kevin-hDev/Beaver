import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { normalizeReasoningMode, reasoningModeOptions } from "@/lib/reasoning-modes";
import {
  mapOAuthModels, mapOAuthResponse, useAvailableModels, withoutInteractiveOnlyModels,
} from "../use-available-models";
import { mapCloudModelSettlements } from "../cloud-models";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

beforeEach(() => {
  vi.clearAllMocks();
});

describe("OAuth models", () => {
  it("conserve un refus sûr du catalogue API sans exposer le détail technique", () => {
    const specs = [{
      id: "mistral",
      display_name: "Mistral",
      category: "llm" as const,
      signup_url: "https://example.invalid",
      connection_kind: "api_key" as const,
    }];
    const result = mapCloudModelSettlements(specs, [{
      status: "rejected",
      reason: "provider_access_unavailable",
    }]);

    expect(result.groups.size).toBe(0);
    expect(result.issues.get("mistral")).toEqual({
      providerName: "Mistral",
      code: "provider_access_unavailable",
    });

    const unknown = mapCloudModelSettlements(specs, [{
      status: "rejected",
      reason: "private upstream response",
    }]);
    expect(unknown.issues.get("mistral")?.code).toBe("model_catalog_unavailable");
  });

  it("accepte un provider futur uniquement depuis ses métadonnées publiques", () => {
    const groups = mapOAuthModels([{
      id: "model-v1",
      provider_id: "provider-fictif",
      connection_id: "provider-fictif-oauth",
      provider_display_name: "Provider fictif",
      display_name: "Model V1",
      context_length: 64000,
      supports_tools: true,
      supports_vision: false,
      supports_thinking: true,
      supports_fast_mode: false,
      reasoning_modes: ["low", "high"],
      reasoning_contract: {
        mandatory: true,
        default_effort: "minimal",
        control: { kind: "efforts", efforts: ["minimal", "high"] },
      },
      default_reasoning_mode: "low",
      context_usage_includes_reasoning: false,
      interactive_only: false,
    }]);

    expect(groups.get("provider-fictif-oauth")?.[0]).toMatchObject({
      provider_name: "Provider fictif · OAuth",
      context_length: 64000,
      reasoning_modes: ["low", "high"],
      reasoning_contract: {
        mandatory: true,
        default_effort: "minimal",
        control: { kind: "efforts", efforts: ["minimal", "high"] },
      },
      context_usage_includes_reasoning: false,
    });
    const model = groups.get("provider-fictif-oauth")?.[0];
    expect(reasoningModeOptions(model ?? null).map((entry) => entry.mode))
      .toEqual(["minimal", "high"]);
  });

  it("utilise des ids et libellés distincts des providers API", () => {
    const groups = mapOAuthModels([
      { id: "kimi-for-coding", provider_id: "moonshot", connection_id: "moonshot-oauth", provider_display_name: "Moonshot AI", display_name: "Kimi", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, context_usage_includes_reasoning: true, interactive_only: true },
      { id: "grok-4.3", provider_id: "xai", connection_id: "xai-oauth", provider_display_name: "xAI", display_name: "Grok 4.3", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, context_usage_includes_reasoning: true, interactive_only: true },
      { id: "gpt-5.6-sol", provider_id: "openai", connection_id: "codex-oauth", provider_display_name: "OpenAI", display_name: "gpt-5.6-sol", context_length: 258400, supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: true, context_usage_includes_reasoning: false, interactive_only: false },
    ]);

    expect(groups.get("moonshot-oauth")?.[0].provider_name).toBe("Moonshot AI · OAuth");
    expect(groups.get("xai-oauth")?.[0].provider_name).toBe("xAI · OAuth");
    expect(groups.get("codex-oauth")?.[0].provider_name).toBe("OpenAI · OAuth");
    expect(groups.get("codex-oauth")?.[0].hint).toBe("258K ctx");
    expect(groups.get("codex-oauth")?.[0].supports_fast_mode).toBe(true);
    expect(groups.has("moonshot")).toBe(false);
    expect(groups.has("xai")).toBe(false);
    expect(groups.get("moonshot-oauth")?.[0].interactive_only).toBe(true);

    const automated = withoutInteractiveOnlyModels(groups);
    expect(automated.has("moonshot-oauth")).toBe(false);
    expect(automated.has("xai-oauth")).toBe(false);
    expect(automated.has("codex-oauth")).toBe(true);
  });

  it("conserve les erreurs de catalogue sûres sans inventer de modèle", () => {
    const result = mapOAuthResponse({
      models: [],
      issues: [{ provider_id: "moonshot", code: "moonshot_membership_unverified" }],
    });

    expect(result.groups.has("moonshot-oauth")).toBe(false);
    expect(result.issues.get("moonshot")).toBe("moonshot_membership_unverified");
  });

  it("conserve le nom officiel et les efforts de K3", () => {
    const groups = mapOAuthModels([{
      id: "k3",
      display_name: "K3",
      provider_id: "moonshot",
      connection_id: "moonshot-oauth",
      provider_display_name: "Moonshot AI",
      supports_tools: true,
      supports_vision: true,
      supports_thinking: true,
      supports_fast_mode: false,
      reasoning_modes: ["low", "high", "max"],
      default_reasoning_mode: "max",
      context_usage_includes_reasoning: true,
      interactive_only: true,
    }]);

    const model = groups.get("moonshot-oauth")?.[0];
    expect(model?.id).toBe("k3");
    expect(model?.display_name).toBe("K3");
    expect(model?.reasoning_modes).toEqual(["low", "high", "max"]);
    expect(model?.default_reasoning_mode).toBe("max");
  });

  it("préserve le catalogue Codex rempli pour Astra sans le fabriquer si absent", () => {
    const response = mapOAuthResponse({
      models: [{
        id: "gpt-6-astra",
        provider_id: "openai",
        connection_id: "codex-oauth",
        provider_display_name: "OpenAI",
        display_name: "GPT-6 Astra",
        context_length: 1050000,
        supports_tools: true,
        supports_vision: true,
        supports_thinking: true,
        supports_fast_mode: false,
        reasoning_modes: ["low", "medium", "high", "xhigh", "max"],
        default_reasoning_mode: "medium",
        context_usage_includes_reasoning: false,
        interactive_only: false,
      }],
      issues: [],
    });
    const astra = response.groups.get("codex-oauth")?.[0];
    expect(astra?.reasoning_modes).toEqual(["low", "medium", "high", "xhigh", "max"]);
    expect(astra?.default_reasoning_mode).toBe("medium");
    expect(mapOAuthResponse({ models: [], issues: [] }).groups.has("codex-oauth")).toBe(false);
  });

  it("relie le catalogue rempli aux options du sélecteur pour les neuf couples", async () => {
    const cloudSpecs = ["google", "zai", "openai", "openrouter", "qwen"].map((id) => ({
      id,
      display_name: id,
      category: "llm" as const,
      signup_url: "https://example.invalid",
      connection_kind: id === "qwen" ? "qwen_model_studio" as const : "api_key" as const,
    }));
    const cloudModels = {
      google: [{ id: "gemini-3.8-flash", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, reasoning_modes: ["low", "medium", "high"] as const, default_reasoning_mode: "medium" as const, context_usage_includes_reasoning: true }],
      zai: [{ id: "glm-5.3-flash", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, reasoning_modes: ["low", "high", "max"] as const, default_reasoning_mode: "max" as const, context_usage_includes_reasoning: true }],
      openai: [{ id: "gpt-6-astra", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, reasoning_modes: ["low", "medium", "high", "xhigh", "max"] as const, context_usage_includes_reasoning: true }],
      openrouter: [
        { id: "google/gemini-3.8-flash", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, supported_parameters: ["tools"], reasoning_modes: ["low", "medium", "high"] as const, default_reasoning_mode: "medium" as const, context_usage_includes_reasoning: true },
        { id: "z-ai/glm-5.3-flash", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, reasoning_modes: ["low", "high", "max"] as const, default_reasoning_mode: "max" as const, context_usage_includes_reasoning: true },
        { id: "openai/gpt-6-astra", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, reasoning_modes: ["low", "medium", "high", "xhigh", "max"] as const, context_usage_includes_reasoning: true },
      ],
      // Synthetic metadata only: auto is NOT an established Alibaba contract or activation.
      qwen: [{ id: "ZHIPU/GLM-5.3-Flash", supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false, reasoning_modes: ["auto"] as const, default_reasoning_mode: "auto" as const, context_usage_includes_reasoning: true }],
    };
    const ollamaModel = {
      name: "glm-5.3-flash:cloud", size: 0, family: "glm", parameter_size: "cloud",
      quantization: "unknown", architecture: "cloud", is_moe: false, context_length: 1048576,
      capabilities: ["thinking", "tools"], reasoning_modes: ["low", "high", "max"],
      default_reasoning_mode: "max", context_usage_includes_reasoning: true,
      digest_short: "fixture", aliases: [], is_customized: false,
    };
    const codexModel = {
      id: "gpt-6-astra", provider_id: "openai", connection_id: "codex-oauth",
      provider_display_name: "OpenAI", display_name: "GPT-6 Astra", context_length: 1050000,
      supports_tools: true, supports_vision: true, supports_thinking: true, supports_fast_mode: false,
      reasoning_modes: ["low", "medium", "high", "xhigh", "max"], default_reasoning_mode: "medium",
      context_usage_includes_reasoning: false, interactive_only: false,
    };

    vi.mocked(invoke).mockImplementation((command, args) => {
      if (command === "list_ollama_models") return Promise.resolve([ollamaModel]);
      if (command === "list_llm_providers_catalog") return Promise.resolve(cloudSpecs);
      if (command === "list_configured_providers") return Promise.resolve(cloudSpecs.map((spec) => spec.id));
      if (command === "list_llm_models") {
        const provider = (args as { providerId?: string } | undefined)?.providerId as keyof typeof cloudModels;
        return Promise.resolve(cloudModels[provider] ?? []);
      }
      if (command === "list_oauth_provider_models") return Promise.resolve({ models: [codexModel], issues: [] });
      return Promise.reject(new Error(`unexpected command: ${command}`));
    });

    const { result } = renderHook(() => useAvailableModels());
    await waitFor(() => expect(result.current.loading).toBe(false));
    const expected = [
      ["google", "gemini-3.8-flash", ["low", "medium", "high"], "medium"],
      ["zai", "glm-5.3-flash", ["low", "high", "max"], "max"],
      ["openai", "gpt-6-astra", ["low", "medium", "high", "xhigh", "max"], "medium"],
      ["openrouter", "google/gemini-3.8-flash", ["low", "medium", "high"], "medium"],
      ["openrouter", "z-ai/glm-5.3-flash", ["low", "high", "max"], "max"],
      ["openrouter", "openai/gpt-6-astra", ["low", "medium", "high", "xhigh", "max"], "medium"],
      ["qwen", "ZHIPU/GLM-5.3-Flash", ["auto"], "auto"],
      ["ollama", "glm-5.3-flash:cloud", ["low", "high", "max"], "max"],
      ["codex-oauth", "gpt-6-astra", ["low", "medium", "high", "xhigh", "max"], "medium"],
    ] as const;
    for (const [provider, id, modes, preferred] of expected) {
      const available = result.current.groups.get(provider)?.find((model) => model.id === id);
      expect(available, `${provider}/${id}`).toBeDefined();
      const options = reasoningModeOptions(available ?? null);
      expect(options.map((entry) => entry.mode)).toEqual(modes);
      expect(normalizeReasoningMode("off", options, available?.default_reasoning_mode)).toBe(preferred);
    }
    expect(result.current.groups.get("openrouter")?.[0].supported_parameters)
      .toEqual(["tools"]);

    // A changed account default must reach the selector, not be replaced by medium.
    codexModel.default_reasoning_mode = "high";
    const oauthChanged = vi.mocked(listen).mock.calls
      .find(([event]) => event === "oauth-provider-status-changed")?.[1];
    expect(oauthChanged).toBeDefined();
    act(() => { oauthChanged?.({ event: "oauth-provider-status-changed", id: 1, payload: null }); });
    await waitFor(() => expect(result.current.groups.get("codex-oauth")?.[0]
      ?.default_reasoning_mode).toBe("high"));
    const refreshed = result.current.groups.get("codex-oauth")?.[0];
    expect(normalizeReasoningMode("off", reasoningModeOptions(refreshed ?? null),
      refreshed?.default_reasoning_mode)).toBe("high");

    vi.mocked(invoke).mockImplementation((command) => Promise.resolve(
      command === "list_oauth_provider_models" ? { models: [], issues: [] } : [],
    ));
    act(() => { oauthChanged?.({ event: "oauth-provider-status-changed", id: 2, payload: null }); });
    await waitFor(() => expect(result.current.groups.size).toBe(0));
    vi.mocked(invoke).mockRejectedValue(new Error("catalogue indisponible"));
    await act(async () => { await result.current.refresh(); });
    expect(result.current.groups.size).toBe(0);
  });
});
