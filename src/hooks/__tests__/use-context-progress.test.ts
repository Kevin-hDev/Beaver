import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";

const eventListeners = vi.hoisted(() => new Map<string, () => void>());

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((event: string, callback: () => void) => {
    eventListeners.set(event, callback);
    return Promise.resolve(() => eventListeners.delete(event));
  }),
}));

import { useContextProgress } from "@/hooks/use-context-progress";

beforeEach(() => {
  vi.clearAllMocks();
  eventListeners.clear();
});

describe("useContextProgress", () => {
  it("utilise la commande normalisée pour toute route", async () => {
    vi.mocked(invoke).mockResolvedValue(123456);

    const { result } = renderHook(() =>
      useContextProgress("modele-fictif", 0, "provider-fictif"),
    );

    await waitFor(() => expect(result.current.max).toBe(123456));
    expect(invoke).toHaveBeenCalledWith("get_model_context", {
      routeId: "provider-fictif",
      modelId: "modele-fictif",
    });
  });

  it("reste à zéro si le contexte est absent", async () => {
    vi.mocked(invoke).mockResolvedValue(null);

    const { result } = renderHook(() => useContextProgress("inconnu", 0, "route-test"));

    await waitFor(() => expect(invoke).toHaveBeenCalled());
    expect(result.current.max).toBe(0);
  });

  it("relit le contexte quand le modèle change", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(8192).mockResolvedValueOnce(16384);
    const { result, rerender } = renderHook(
      ({ model }) => useContextProgress(model, 0, "ollama"),
      { initialProps: { model: "local-a" } },
    );
    await waitFor(() => expect(result.current.max).toBe(8192));

    rerender({ model: "local-b" });

    await waitFor(() => expect(result.current.max).toBe(16384));
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("ne relit pas le contexte quand seul le compteur de génération change", async () => {
    vi.mocked(invoke).mockResolvedValue(8192);
    const { result, rerender } = renderHook(
      ({ used }) => useContextProgress("modele-test", used, "route-test"),
      { initialProps: { used: 0 } },
    );
    await waitFor(() => expect(result.current.max).toBe(8192));

    rerender({ used: 10 });

    expect(result.current.max).toBe(8192);
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("conserve la dernière valeur valable après une erreur de lecture", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(8192).mockRejectedValueOnce(new Error("indisponible"));
    const { result } = renderHook(() =>
      useContextProgress("local", 0, "ollama"),
    );
    await waitFor(() => expect(result.current.max).toBe(8192));

    eventListeners.get("modelfile-updated")?.();

    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
    expect(result.current.max).toBe(8192);
  });

  it("relit la fenêtre effective après une modification du Modelfile", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(128_000).mockResolvedValueOnce(32_000);
    const { result } = renderHook(() => useContextProgress("local", 0, "ollama"));
    await waitFor(() => expect(result.current.max).toBe(128_000));

    eventListeners.get("modelfile-updated")?.();

    await waitFor(() => expect(result.current.max).toBe(32_000));
  });
});
