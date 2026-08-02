import { describe, expect, it } from "vitest";
import { toolsFromMessage } from "./message-tools";
import type { AgentMessage } from "@/types/agent";

function message(): AgentMessage {
  return {
    id: "message",
    role: "assistant",
    content: "",
    files: [],
    timestamp: "2026-08-03T00:00:00Z",
  };
}

describe("toolsFromMessage", () => {
  it("préfère la copie segmentée quand les deux formats sont présents", () => {
    const value = message();
    value.tool_activities = [{ name: "legacy", summary: "" }];
    value.segments = [{
      content: "",
      tools: [{ name: "segment", summary: "" }],
    }];

    expect(toolsFromMessage(value).map((tool) => tool.name)).toEqual(["segment"]);
  });

  it("conserve la lecture des anciennes sessions", () => {
    const value = message();
    value.tool_activities = [{ name: "legacy", summary: "" }];

    expect(toolsFromMessage(value).map((tool) => tool.name)).toEqual(["legacy"]);
  });

  it("conserve l'ancien format quand les segments ne portent aucun outil", () => {
    const value = message();
    value.tool_activities = [{ name: "legacy", summary: "" }];
    value.segments = [{ content: "texte", tools: [] }];

    expect(toolsFromMessage(value).map((tool) => tool.name)).toEqual(["legacy"]);
  });
});
