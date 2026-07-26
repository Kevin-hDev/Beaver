import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useOllamaConnectionRetry } from "@/hooks/use-ollama-connection-retry";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

afterEach(() => {
  vi.useRealTimers();
  vi.mocked(invoke).mockReset();
});

describe("useOllamaConnectionRetry", () => {
  it("affiche l'indicateur puis relance quand Ollama revient", async () => {
    const onRetry = vi.fn();
    vi.mocked(invoke).mockResolvedValue(true);

    const { result } = renderHook(() => useOllamaConnectionRetry({
      error: "errors.ollamaConnectionLost",
      isConnectionError: true,
      isStreaming: false,
      onRetry,
    }));

    await waitFor(() => expect(onRetry).toHaveBeenCalledTimes(1));
    expect(result.current.indicator).toBeNull();
    expect(result.current.suppressError).toBe(true);
    expect(invoke).toHaveBeenCalledWith("is_ollama_running");
  });

  it("s'arrête après 10 tentatives sans relancer", async () => {
    vi.useFakeTimers();
    const onRetry = vi.fn();
    vi.mocked(invoke).mockResolvedValue(false);

    const { result } = renderHook(() => useOllamaConnectionRetry({
      error: "errors.ollamaConnectionLost",
      isConnectionError: true,
      isStreaming: false,
      onRetry,
    }));

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(result.current.indicator?.attempt).toBe(1);
    for (let i = 0; i < 9; i += 1) {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(4000);
      });
    }

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(result.current.indicator).toBeNull();
    expect(result.current.suppressError).toBe(false);
    expect(onRetry).not.toHaveBeenCalled();
  });
});
