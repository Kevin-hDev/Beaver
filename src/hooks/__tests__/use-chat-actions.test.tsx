import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useChatActions } from "../use-chat-actions";

describe("useChatActions", () => {
  it("n'envoie pas le message initial d'une session en lecture seule", async () => {
    const sendMessage = vi.fn().mockResolvedValue(undefined);

    renderHook(() => useChatActions({
      readOnly: true,
      chat: { messages: [], sendMessage, reload: vi.fn(), isStreaming: false },
      sessionId: "child-session",
      initialMessage: "message hérité du parent",
      fileDrop: { addByPaths: vi.fn() },
    }));

    await act(async () => Promise.resolve());

    expect(sendMessage).not.toHaveBeenCalled();
  });
});
