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

  it("rattache les artefacts persistés du résultat à l'appel d'outil", () => {
    const assistant: AgentMessage = {
      ...message("assistant", ""),
      tool_calls: [{
        id: "call-a",
        function: { name: "extension_tool", arguments: { input: "report" } },
      }],
    };
    const result: AgentMessage = {
      ...message("tool", "done"),
      tool_name: "extension_tool",
      tool_call_id: "call-a",
      tool_activities: [{
        name: "extension_tool",
        summary: "",
        artifacts: [{
          name: "report.txt",
          mime_type: "text/plain",
          bytes: 3,
          sha256: "a".repeat(64),
          purpose: "artifact",
          source: { kind: "workspace_file", path: "/workspace/report.txt" },
          verification: "intact",
        }],
      }],
    };

    const normalized = normalizeSavedToolHistory([
      message("user", "Create"), assistant, result, message("assistant", "Ready"),
    ]);

    expect(normalized[1].segments?.[0].tools[0].artifacts?.[0]).toMatchObject({
      name: "report.txt",
      verification: "intact",
    });
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
