import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useUnavailableModelFallback } from "../use-unavailable-model-fallback";
import type { AvailableModel } from "../use-available-models";

describe("useUnavailableModelFallback", () => {
  it("ne remplace pas automatiquement le modèle d'une session en lecture seule", async () => {
    const updateModel = vi.fn().mockResolvedValue(undefined);
    const availableModels = new Map<string, AvailableModel[]>([
      ["ollama", [{ id: "available", provider_id: "ollama" } as AvailableModel]],
    ]);

    renderHook(() => useUnavailableModelFallback({
      enabled: false,
      availableModels,
      model: "missing",
      provider: "ollama",
      activeSessionId: "child-session",
      updateModel,
      setWelcomeModel: vi.fn(),
    }));

    await act(async () => Promise.resolve());

    expect(updateModel).not.toHaveBeenCalled();
  });
});
