import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useStartupGate } from "./use-startup-gate";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("useStartupGate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("affiche l'onboarding avant Ollama au premier lancement", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "is_ollama_installed") return Promise.resolve(false);
      if (command === "get_advanced_settings") {
        return Promise.resolve({
          onboarding_completed: false,
          ollama_setup_skipped: false,
        });
      }
      return Promise.resolve();
    });

    const { result } = renderHook(() => useStartupGate());

    await waitFor(() => expect(result.current.view).toBe("onboarding"));
    expect(result.current.showOllamaSetup).toBe(true);
  });

  it("termine onboarding et Ollama dans le meme patch", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "is_ollama_installed") return Promise.resolve(false);
      if (command === "get_advanced_settings") {
        return Promise.resolve({
          onboarding_completed: true,
          ollama_setup_skipped: false,
        });
      }
      return Promise.resolve();
    });

    const { result } = renderHook(() => useStartupGate());
    await waitFor(() => expect(result.current.view).toBe("ollama"));

    await act(async () => {
      await result.current.skipOllamaSetup();
    });

    expect(invoke).toHaveBeenCalledWith("patch_advanced_settings", {
      patch: {
        onboarding_completed: true,
        ollama_setup_skipped: true,
      },
    });
    expect(result.current.view).toBe("app");
  });
});
