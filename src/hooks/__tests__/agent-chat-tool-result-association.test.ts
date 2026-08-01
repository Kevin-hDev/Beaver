import { describe, expect, it } from "vitest";
import { applyToolResult } from "@/hooks/agent-chat-tool-results";
import type { ToolActivity } from "@/hooks/agent-chat-utils";

describe("association des résultats d'outils", () => {
  it("utilise l'index pour départager un identifiant dupliqué", () => {
    const tools: ToolActivity[] = [
      { name: "grep", args: {}, callId: "duplicate", callIndex: 2 },
      { name: "grep", args: {}, callId: "duplicate", callIndex: 3 },
    ];

    const applied = applyToolResult(tools, {
      name: "grep",
      callId: "duplicate",
      callIndex: 3,
      content: "second",
      isError: false,
    });

    expect(applied.appliedIndex).toBe(1);
    expect(applied.tools[0].result).toBeUndefined();
    expect(applied.tools[1].result).toBe("second");
  });

  it("refuse des métadonnées stables entièrement ambiguës", () => {
    const tools: ToolActivity[] = [
      { name: "grep", args: {}, callId: "duplicate", callIndex: 2 },
      { name: "grep", args: {}, callId: "duplicate", callIndex: 2 },
    ];

    const applied = applyToolResult(tools, {
      name: "grep",
      callId: "duplicate",
      callIndex: 2,
      content: "ambiguous",
      isError: false,
    });

    expect(applied.appliedIndex).toBe(-1);
    expect(applied.tools.every((tool) => tool.result === undefined)).toBe(true);
  });
});
