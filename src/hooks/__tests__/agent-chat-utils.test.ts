import { describe, it, expect } from "vitest";
import {
  toolsToRecords,
  segmentsToRecords,
  buildSegmentedMessage,
  expandToolActivities,
  expandSegmentsToChat,
} from "@/hooks/agent-chat-utils";
import type { ToolActivity, StreamSegment, SavedSegment } from "@/hooks/agent-chat-utils";

function tool(name: string, args: Record<string, unknown>, result?: string): ToolActivity {
  return { name, args, result };
}

describe("toolsToRecords", () => {
  it("bash — summary = command", () => {
    const [r] = toolsToRecords([tool("bash", { command: "ls -la" })]);
    expect(r.summary).toBe("ls -la");
  });

  it("conserve la dernière sortie live si le stream est interrompu", () => {
    const [record] = toolsToRecords([{
      name: "bash",
      args: { command: "build" },
      liveOutput: "compilation...",
    }]);

    expect(record.result).toBe("compilation...");
  });

  it("read_file — summary = path", () => {
    const [r] = toolsToRecords([tool("read_file", { path: "/tmp/foo.txt" })]);
    expect(r.summary).toBe("/tmp/foo.txt");
  });

  it("edit_file — extrait old_text et new_text", () => {
    const [r] = toolsToRecords([tool("edit_file", { path: "/f", old_string: "avant", new_string: "après" })]);
    expect(r.old_text).toBe("avant");
    expect(r.new_text).toBe("après");
  });

  it("write_file — extrait content", () => {
    const [r] = toolsToRecords([tool("write_file", { path: "/f", content: "hello world" })]);
    expect(r.content).toBe("hello world");
  });

  it("grep — summary = pattern", () => {
    const [r] = toolsToRecords([tool("grep", { pattern: "TODO" })]);
    expect(r.summary).toBe("TODO");
  });

  it("web_search — summary = query", () => {
    const [r] = toolsToRecords([tool("web_search", { query: "vitest mock" })]);
    expect(r.summary).toBe("vitest mock");
  });

  it("outil inconnu — summary = JSON tronqué", () => {
    const [r] = toolsToRecords([tool("custom_tool", { foo: "bar" })]);
    expect(r.summary).toContain("bar");
    expect(r.summary.length).toBeLessThanOrEqual(80);
  });

  it("conserve uniquement le nom lisible du skill", () => {
    const [record] = toolsToRecords([{
      name: "load_skill",
      args: { skill_id: "local:skill:0123456789abcdef" },
      displaySummary: "context7-docs",
    }]);

    expect(record.summary).toBe("context7-docs");
    expect(record.summary).not.toContain("skill_id");
  });

  it("ne sauvegarde jamais l'identifiant opaque d'un skill sans nom résolu", () => {
    const [record] = toolsToRecords([tool(
      "load_skill",
      { skill_id: "local:skill:0123456789abcdef" },
    )]);

    expect(record.summary).toBe("");
  });

  it("conserve le numéro de ligne structuré", () => {
    const [r] = toolsToRecords([{ name: "edit_file", args: { path: "/f", old_string: "", new_string: "" }, startLine: 42 }]);
    expect(r.start_line).toBe(42);
  });

  it("write_spreadsheet — extrait operations comme content", () => {
    const ops = [{ type: "set", cell: "A1", value: "x" }];
    const [r] = toolsToRecords([tool("write_spreadsheet", { path: "/s.xlsx", operations: ops })]);
    expect(r.content).toBe(JSON.stringify(ops));
  });
});

describe("segmentsToRecords", () => {
  it("combine les tools de tous les segments", () => {
    const segments: StreamSegment[] = [
      { thinking: "", tools: [tool("bash", { command: "ls" })], content: "a" },
      { thinking: "", tools: [tool("grep", { pattern: "TODO" })], content: "b" },
    ];
    const records = segmentsToRecords(segments);
    expect(records).toHaveLength(2);
    expect(records[0].name).toBe("bash");
    expect(records[1].name).toBe("grep");
  });
});

describe("buildSegmentedMessage", () => {
  it("joint les contenus avec \\n\\n", () => {
    const segments: StreamSegment[] = [
      { thinking: "", tools: [], content: "premier" },
      { thinking: "", tools: [], content: "second" },
    ];
    const { content } = buildSegmentedMessage(segments);
    expect(content).toBe("premier\n\nsecond");
  });

  it("joint les thinkings", () => {
    const segments: StreamSegment[] = [
      { thinking: "pensée 1", tools: [], content: "" },
      { thinking: "pensée 2", tools: [], content: "" },
    ];
    const { thinking } = buildSegmentedMessage(segments);
    expect(thinking).toBe("pensée 1\n\npensée 2");
  });

  it("retourne undefined pour toolRecords si aucun outil", () => {
    const segments: StreamSegment[] = [{ thinking: "", tools: [], content: "texte" }];
    const { toolRecords } = buildSegmentedMessage(segments);
    expect(toolRecords).toBeUndefined();
  });

  it("conserve la phase des segments sauvegardés", () => {
    const segments: StreamSegment[] = [{ thinking: "", tools: [], content: "travail", phase: "work" }];
    const { segments: savedSegments } = buildSegmentedMessage(segments);
    expect(savedSegments?.[0].phase).toBe("work");
  });
});

describe("expandToolActivities", () => {
  it("crée message assistant + messages tool", () => {
    const activities = [{ name: "bash", summary: "ls", result: "fichiers.txt" }];
    const msgs = expandToolActivities(activities, "réponse finale");
    expect(msgs[0].role).toBe("assistant");
    expect(msgs[0].tool_calls).toHaveLength(1);
    expect(msgs[1].role).toBe("tool");
    expect(msgs[1].content).toBe("fichiers.txt");
    expect(msgs[msgs.length - 1].content).toBe("réponse finale");
  });
});

describe("expandSegmentsToChat", () => {
  it("fallback sur content si pas de segments", () => {
    const msgs = expandSegmentsToChat([], "contenu direct");
    expect(msgs).toHaveLength(1);
    expect(msgs[0].role).toBe("assistant");
    expect(msgs[0].content).toBe("contenu direct");
  });

  it("génère messages tool pour segment avec tools", () => {
    const segments: SavedSegment[] = [
      { content: "", tools: [{ name: "bash", summary: "ls", result: "ok" }] },
    ];
    const msgs = expandSegmentsToChat(segments, "");
    expect(msgs[0].role).toBe("assistant");
    expect(msgs[0].tool_calls).toHaveLength(1);
    expect(msgs[1].role).toBe("tool");
    expect(msgs[1].content).toBe("ok");
  });

  it("génère un message assistant pour segment sans tools", () => {
    const segments: SavedSegment[] = [{ content: "texte pur", tools: [] }];
    const msgs = expandSegmentsToChat(segments, "");
    expect(msgs).toHaveLength(1);
    expect(msgs[0].role).toBe("assistant");
    expect(msgs[0].content).toBe("texte pur");
  });

  it("multiple segments avec et sans tools génèrent la bonne séquence", () => {
    const segments: SavedSegment[] = [
      { content: "intro", tools: [] },
      { content: "", tools: [{ name: "bash", summary: "ls", result: "fichiers" }] },
      { content: "conclusion", tools: [] },
    ];
    const msgs = expandSegmentsToChat(segments, "");
    // segment 1 : assistant text
    expect(msgs[0].role).toBe("assistant");
    expect(msgs[0].content).toBe("intro");
    // segment 2 : assistant tool_calls + tool result
    expect(msgs[1].role).toBe("assistant");
    expect(msgs[1].tool_calls).toHaveLength(1);
    expect(msgs[2].role).toBe("tool");
    expect(msgs[2].content).toBe("fichiers");
    // segment 3 : assistant text
    expect(msgs[3].role).toBe("assistant");
    expect(msgs[3].content).toBe("conclusion");
    expect(msgs).toHaveLength(4);
  });
});
