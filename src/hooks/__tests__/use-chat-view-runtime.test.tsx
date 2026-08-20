import { act, renderHook } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { useChatViewRuntime } from "../use-chat-view-runtime";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("useChatViewRuntime", () => {
  it("ne démarre aucune reprise de connexion en lecture seule", async () => {
    vi.mocked(invoke).mockResolvedValue(null);
    const { result } = renderHook(() => useChatViewRuntime({
      readOnly: true,
      chat: {
        messages: [],
        reload: vi.fn(),
        edit: vi.fn(),
        error: "ollama_connection_lost",
        isConnectionError: true,
        isStreaming: false,
      },
      setPreview: vi.fn(),
    }));

    await act(async () => Promise.resolve());

    expect(invoke).not.toHaveBeenCalled();
    expect(result.current.retryIndicator).toBeNull();
    expect(result.current.showError).toBe(true);
  });
});
