import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { useOllamaRuntimeStatus } from "./use-ollama-runtime-status";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const readyStatus = {
  bundle: "ready",
  daemon: { owned: { endpoint: { port: 11434 } } },
  operation: "idle",
  progress: null,
  last_error: null,
} as const;

describe("useOllamaRuntimeStatus", () => {
  it("reads the complete typed status and exposes a refresh action", async () => {
    vi.mocked(invoke).mockResolvedValue(readyStatus);
    const { result } = renderHook(() => useOllamaRuntimeStatus());

    await waitFor(() => expect(result.current.status).toEqual(readyStatus));
    expect(invoke).toHaveBeenCalledWith("get_ollama_runtime_status");

    await act(async () => { await result.current.refresh(); });
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("keeps the status unavailable when the read fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("path /tmp and stack"));
    const { result } = renderHook(() => useOllamaRuntimeStatus());

    await waitFor(() => expect(result.current.readError).toBe(true));
    expect(result.current.status).toBeNull();
    expect(result.current.loading).toBe(false);
  });
});
