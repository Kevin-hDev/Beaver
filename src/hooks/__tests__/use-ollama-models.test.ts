import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useOllamaModels } from "../use-ollama-models";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe("useOllamaModels", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("ne liste pas les modeles quand le hook est desactive", async () => {
    const { result } = renderHook(() => useOllamaModels({ enabled: false }));

    await result.current.refresh();

    expect(result.current.models).toEqual([]);
    expect(result.current.loading).toBe(false);
    expect(invoke).not.toHaveBeenCalled();
    expect(listen).not.toHaveBeenCalled();
  });

  it("liste les modeles quand le hook est actif", async () => {
    vi.mocked(invoke).mockResolvedValue([{ name: "llama3" }]);

    const { result } = renderHook(() => useOllamaModels({ enabled: true }));

    await waitFor(() => expect(result.current.models).toEqual([{ name: "llama3" }]));
    expect(invoke).toHaveBeenCalledWith("list_ollama_models");
  });

  it("conserve les modes exacts du modèle cloud et reflète un catalogue vide ou indisponible", async () => {
    const cloudModel = {
      name: "glm-5.3-flash:cloud",
      size: 0,
      family: "glm",
      parameter_size: "cloud",
      quantization: "unknown",
      architecture: "cloud",
      is_moe: false,
      context_length: 1048576,
      capabilities: ["thinking", "tools"] as const,
      reasoning_modes: ["low", "high", "max"] as const,
      default_reasoning_mode: "max" as const,
      context_usage_includes_reasoning: true,
      digest_short: "fixture",
      aliases: [],
      is_customized: false,
    };
    vi.mocked(invoke).mockResolvedValue([cloudModel]);

    const { result } = renderHook(() => useOllamaModels({ enabled: true }));
    await waitFor(() => expect(result.current.models[0]?.name).toBe("glm-5.3-flash:cloud"));
    expect(result.current.models[0]?.reasoning_modes).toEqual(["low", "high", "max"]);
    expect(result.current.models[0]?.default_reasoning_mode).toBe("max");

    vi.mocked(invoke).mockResolvedValueOnce([]);
    await act(async () => { await result.current.refresh(); });
    expect(result.current.models).toEqual([]);

    vi.mocked(invoke).mockRejectedValueOnce(new Error("catalogue indisponible"));
    await act(async () => { await result.current.refresh(); });
    expect(result.current.models).toEqual([]);
  });
});
