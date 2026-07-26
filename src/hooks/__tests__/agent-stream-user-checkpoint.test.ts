import { describe, expect, it } from "vitest";
import { createManagedStreamState } from "../agent-chat-stream-types";
import { checkpointQueuedUserMessages } from "../agent-stream-user-checkpoint";
import type { AgentMessage } from "@/types/agent";

describe("active stream user checkpoint", () => {
  it("reste rattaché au même flux live jusqu'à sa vraie fin", () => {
    const state = createManagedStreamState([user("u1", "Question")], 0);
    state.streamStartedAt = 123;
    state.liveTokenCount = 7;
    state.currentContent = "Travail déjà visible";
    state.currentContentPhase = "work";
    state.queuedUserMessages = [user("u2", "Ça va ?")];

    const result = checkpointQueuedUserMessages(state);
    const checkpoint = result?.state.messages[1] as AgentMessage & {
      is_stream_checkpoint?: boolean;
    };

    expect(result?.state.streamStartedAt).toBe(123);
    expect(result?.state.liveTokenCount).toBe(7);
    expect(checkpoint.is_stream_checkpoint).toBe(true);
  });
});

function user(id: string, content: string): AgentMessage {
  return {
    id, role: "user", content, files: [], timestamp: "2026-07-12T10:00:00Z",
  };
}
