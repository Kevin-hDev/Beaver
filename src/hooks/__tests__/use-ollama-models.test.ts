import { renderHook, waitFor } from "@testing-library/react";
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
});
