import { describe, expect, it } from "vitest";
import { toVisibleMessageInput } from "../agent-visible-message-input";
import type { AgentMessage } from "@/types/agent";

describe("toVisibleMessageInput", () => {
  it("ne transmet que le DTO visible du shim temporaire", () => {
    const message: AgentMessage = {
      id: "assistant-1",
      turn_id: "turn-owned-by-rust",
      role: "assistant",
      content: "Réponse visible",
      thinking: "Raisonnement visible",
      files: [],
      timestamp: "2026-08-25T10:00:00Z",
      tokens: 7,
      reasoning_replay_status: "preserved",
      is_stream_checkpoint: true,
      tool_calls: [{
        id: "provider-call-1",
        function: { name: "inspect", arguments: { visible: true } },
      }],
    };

    expect(toVisibleMessageInput(message)).toEqual({
      id: "assistant-1",
      role: "assistant",
      content: "Réponse visible",
      thinking: "Raisonnement visible",
      tool_calls: [{
        id: "provider-call-1",
        function: { name: "inspect", arguments: { visible: true } },
      }],
      tool_name: undefined,
      tool_call_id: undefined,
      tool_activities: undefined,
      segments: undefined,
      files: [],
      timestamp: "2026-08-25T10:00:00Z",
      tokens: 7,
      work_duration_ms: undefined,
      skill_names: undefined,
      stream_run_id: undefined,
      stream_part: undefined,
    });
  });
});
