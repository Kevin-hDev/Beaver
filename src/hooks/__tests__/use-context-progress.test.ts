import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

import { useContextProgress } from "@/hooks/use-context-progress";

beforeEach(() => {
  vi.clearAllMocks();
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

  it("rafraîchit le contexte après une nouvelle consommation", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(8192).mockResolvedValueOnce(16384);
    const { result, rerender } = renderHook(
      ({ used }) => useContextProgress("modele-test", used, "route-test"),
      { initialProps: { used: 0 } },
    );
    await waitFor(() => expect(result.current.max).toBe(8192));

    rerender({ used: 10 });

    await waitFor(() => expect(result.current.max).toBe(16384));
  });
});
