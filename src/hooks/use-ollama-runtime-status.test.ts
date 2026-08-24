import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useOllamaRuntimeStatus } from "./use-ollama-runtime-status";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

const readyStatus = {
  bundle: "ready",
  daemon: { owned: { endpoint: { port: 11434 } } },
  operation: "idle",
  progress: null,
  last_error: null,
} as const;

describe("useOllamaRuntimeStatus", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("reads the complete typed status and exposes a refresh action", async () => {
    vi.mocked(listen).mockResolvedValue(vi.fn());
    vi.mocked(invoke).mockResolvedValue(readyStatus);
    const { result } = renderHook(() => useOllamaRuntimeStatus());

    await waitFor(() => expect(result.current.status).toEqual(readyStatus));
    expect(invoke).toHaveBeenCalledWith("get_ollama_runtime_status");

    await act(async () => { await result.current.refresh(); });
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("keeps the status unavailable when the read fails", async () => {
    vi.mocked(listen).mockResolvedValue(vi.fn());
    vi.mocked(invoke).mockRejectedValue(new Error("path /tmp and stack"));
    const { result } = renderHook(() => useOllamaRuntimeStatus());

    await waitFor(() => expect(result.current.readError).toBe(true));
    expect(result.current.status).toBeNull();
    expect(result.current.loading).toBe(false);
  });

  it("rejects an incomplete status instead of exposing an unsafe value", async () => {
    vi.mocked(listen).mockResolvedValue(vi.fn());
    vi.mocked(invoke).mockResolvedValue({ bundle: "ready" });
    const { result } = renderHook(() => useOllamaRuntimeStatus());

    await waitFor(() => expect(result.current.readError).toBe(true));
    expect(result.current.status).toBeNull();
  });

  it("refreshes when Ollama becomes ready after the initial read", async () => {
    let onStatus: ((event: { payload: boolean }) => void) | undefined;
    vi.mocked(listen).mockImplementation((_event, handler) => {
      onStatus = handler as (event: { payload: boolean }) => void;
      return Promise.resolve(() => undefined);
    });
    vi.mocked(invoke)
      .mockResolvedValueOnce({ ...readyStatus, daemon: "unavailable" })
      .mockResolvedValueOnce(readyStatus);

    const { result } = renderHook(() => useOllamaRuntimeStatus());
    await waitFor(() => expect(result.current.status?.daemon).toBe("unavailable"));

    act(() => { onStatus?.({ payload: true }); });

    await waitFor(() => expect(result.current.status).toEqual(readyStatus));
    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
