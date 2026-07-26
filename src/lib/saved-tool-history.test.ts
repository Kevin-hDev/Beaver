import { describe, expect, it } from "vitest";
import { normalizeSavedToolHistory } from "./saved-tool-history";
import type { AgentMessage } from "@/types/agent";

describe("normalizeSavedToolHistory", () => {
  it("range un thinking enfant sans outil dans la phase de travail", () => {
    const normalized = normalizeSavedToolHistory([
      message("user", "Mission"),
      { ...message("assistant", "Rapport final"), thinking: "Analyse détaillée" },
    ]);

    expect(normalized).toHaveLength(2);
    expect(normalized[1].segments).toEqual([
      { thinking: "Analyse détaillée", content: "", tools: [], phase: "work" },
      { content: "Rapport final", tools: [], phase: "final" },
    ]);
  });

  it("ne modifie pas une timeline parent déjà segmentée", () => {
    const parent = {
      ...message("assistant", "Final"),
      segments: [{ content: "Final", tools: [], phase: "final" as const }],
    };

    expect(normalizeSavedToolHistory([parent])).toEqual([parent]);
  });
});

function message(role: AgentMessage["role"], content: string): AgentMessage {
  return {
    id: `${role}-${content}`,
    role,
    content,
    files: [],
    timestamp: "2026-07-13T12:00:00Z",
  };
}
