import { describe, expect, it } from "vitest";
import { buildSegmentedMessage, toolsToRecords } from "@/hooks/agent-chat-utils";
import type { StreamSegment, ToolActivity } from "@/hooks/agent-chat-utils";

function tool(name: string, args: Record<string, unknown>, result?: string): ToolActivity {
  return { name, args, result };
}

describe("toolsToRecords — cas limites", () => {
  it("args non-string pour path (number) → summary fallback vide", () => {
    const args: Record<string, unknown> = { path: 42 };
    const [record] = toolsToRecords([tool("read_file", args)]);
    expect(record.summary).toBe("");
  });

  it("args manquants complètement (objet vide) → summary vide", () => {
    const [record] = toolsToRecords([tool("bash", {})]);
    expect(record.summary).toBe("");
  });

  it("list_dir sans path → summary = '.'", () => {
    const [record] = toolsToRecords([tool("list_dir", {})]);
    expect(record.summary).toBe(".");
  });

  it("laisse start_line vide quand le backend ne le fournit pas", () => {
    const [record] = toolsToRecords([{
      name: "edit_file",
      args: { path: "/f", old_string: "", new_string: "" },
      result: "Succès sans numéro de ligne",
    }]);
    expect(record.start_line).toBeUndefined();
  });
});

describe("buildSegmentedMessage — cas limites", () => {
  it("segments avec contenu vide ne génèrent pas de séparateurs parasites", () => {
    const segments: StreamSegment[] = [
      { thinking: "", tools: [], content: "" },
      { thinking: "", tools: [], content: "seul contenu" },
      { thinking: "", tools: [], content: "" },
    ];
    const { content } = buildSegmentedMessage(segments);
    expect(content).toBe("seul contenu");
    expect(content).not.toContain("\n\n");
  });

  it("segments avec tools retourne des toolRecords définis", () => {
    const segments: StreamSegment[] = [
      { thinking: "", tools: [tool("bash", { command: "echo hello" })], content: "ok" },
    ];
    const { toolRecords } = buildSegmentedMessage(segments);
    expect(toolRecords).toBeDefined();
    expect(toolRecords).toHaveLength(1);
    expect(toolRecords![0].name).toBe("bash");
  });
});
