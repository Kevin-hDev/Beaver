import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useChatClone } from "../use-chat-clone";
import type { AgentMessage } from "@/types/agent";

const messages: AgentMessage[] = [
  baseMessage("m1", "user"),
  baseMessage("m2", "assistant"),
];

describe("useChatClone", () => {
  it("ferme immédiatement le dialogue pour un clone sans résumé", async () => {
    let finishClone!: () => void;
    const onCloneMessage = vi.fn(() => new Promise<void>((resolve) => {
      finishClone = resolve;
    }));
    const { result } = renderHook(() =>
      useChatClone("parent", messages, onCloneMessage));

    act(() => result.current.requestClone("m1"));
    act(() => void result.current.submitClone("cut"));

    expect(result.current.pendingClone).toBeNull();
    expect(result.current.cloneBusy).toBe(false);
    expect(result.current.summaryRun).toBeNull();

    await act(async () => {
      finishClone();
      await Promise.resolve();
    });
  });

  it("réouvre le dialogue si le clone simple échoue", async () => {
    const onCloneMessage = vi.fn(() => Promise.reject(new Error("failed")));
    const { result } = renderHook(() =>
      useChatClone("parent", messages, onCloneMessage));

    act(() => result.current.requestClone("m1"));
    await act(async () => {
      await result.current.submitClone("cut");
    });

    expect(result.current.pendingClone?.messageId).toBe("m1");
    expect(result.current.pendingClone?.error).toBe("failed");
    expect(result.current.cloneBusy).toBe(false);
  });

  it("masque, réouvre puis annule une génération de résumé", async () => {
    let finishClone!: () => void;
    const onCloneMessage = vi.fn(() => new Promise<void>((resolve) => {
      finishClone = resolve;
    }));
    const onCancelCloneSummary = vi.fn(() => Promise.resolve());
    const { result } = renderHook(() =>
      useChatClone("parent", messages, onCloneMessage, onCancelCloneSummary));

    act(() => result.current.requestClone("m1"));
    act(() => void result.current.submitClone("summary"));
    await waitFor(() => expect(result.current.summaryRun?.visible).toBe(true));
    const operationId = result.current.summaryRun?.operationId ?? "";

    act(() => result.current.cancelClone());
    expect(result.current.summaryRun?.visible).toBe(false);

    act(() => result.current.showRunningClone());
    expect(result.current.pendingClone?.messageId).toBe("m1");
    expect(result.current.cloneBusy).toBe(true);

    await act(async () => {
      await result.current.abortClone();
    });
    expect(onCancelCloneSummary).toHaveBeenCalledWith(operationId);
    expect(result.current.summaryRun).toBeNull();

    await act(async () => {
      finishClone();
      await Promise.resolve();
    });
  });
});

function baseMessage(id: string, role: "user" | "assistant"): AgentMessage {
  return {
    id,
    role,
    content: "message",
    timestamp: new Date().toISOString(),
    files: [],
  };
}
