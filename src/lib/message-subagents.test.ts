import { describe, expect, it } from "vitest";
import { collectMessageSubagents } from "./message-subagents";
import type { AgentMessage, SubagentInfo, ToolActivityRecord } from "@/types/agent";

describe("collectMessageSubagents", () => {
  it("extrait un sous-agent depuis un delegate_task", () => {
    const agents = collectMessageSubagents(message([delegate("child-a")]), [
      known("child-a", "coder", "completed"),
    ]);

    expect(agents).toHaveLength(1);
    expect(agents[0]).toMatchObject({
      sessionId: "child-a",
      type: "coder",
      status: "completed",
      description: "Implémentation",
    });
  });

  it("ignore les tools hors delegate_task", () => {
    const agents = collectMessageSubagents(message([
      { name: "bash", summary: "npm test", result: "ok" },
    ]), [known("child-a", "explorer", "completed")]);

    expect(agents).toEqual([]);
  });

  it("ignore les segments finaux", () => {
    const agents = collectMessageSubagents({
      ...message([]),
      segments: [
        { content: "", tools: [delegate("child-a")], phase: "work" },
        { content: "Réponse finale", tools: [delegate("child-b")], phase: "final" },
      ],
    }, [
      known("child-a", "explorer", "completed"),
      known("child-b", "coder", "completed"),
    ]);

    expect(agents.map((agent) => agent.sessionId)).toEqual(["child-a"]);
  });

  it("déduplique en conservant l'ordre d'appel", () => {
    const agents = collectMessageSubagents(message([
      delegate("child-b"),
      delegate("child-a"),
      delegate("child-b"),
    ]), [
      known("child-a", "coder", "completed"),
      known("child-b", "explorer", "completed"),
    ]);

    expect(agents.map((agent) => agent.sessionId)).toEqual(["child-b", "child-a"]);
  });

  it("utilise un fallback borné si la session n'est pas encore connue", () => {
    const agents = collectMessageSubagents(message([
      delegate("child-a", {
        subagent_type: "coder",
        prompt: "x".repeat(200),
        description: "Audit",
      }),
    ]));

    expect(agents[0]).toMatchObject({
      sessionId: "child-a",
      type: "coder",
      status: "completed",
      description: "Audit",
    });
    expect(agents[0].promptPreview).toHaveLength(120);
  });

  it("borne l'inspection des gros messages et des gros résultats", () => {
    const hugeResult = `${"x".repeat(2_000)}<subagent id="late-child" state="running" />`;
    const agents = collectMessageSubagents({
      ...message([]),
      segments: [
        {
          content: "",
          tools: [{ ...delegate("visible-child"), result: hugeResult }],
          phase: "work",
        },
        ...Array.from({ length: 400 }, (_, index) => ({
          content: "",
          tools: [delegate(`child-${index}`)],
          phase: "work" as const,
        })),
      ],
    });

    expect(agents).toHaveLength(64);
    expect(agents.some((agent) => agent.sessionId === "late-child")).toBe(false);
    expect(agents.some((agent) => agent.sessionId === "child-399")).toBe(false);
  });
});

function message(tools: ToolActivityRecord[]): AgentMessage {
  return {
    id: "assistant",
    role: "assistant",
    content: "",
    files: [],
    timestamp: new Date(0).toISOString(),
    tool_activities: tools,
  };
}

function delegate(sessionId: string, args: Record<string, unknown> = {}): ToolActivityRecord {
  return {
    name: "delegate_task",
    summary: "delegate",
    args,
    result: `<subagent id="${sessionId}" state="running">ok</subagent>`,
  };
}

function known(
  sessionId: string,
  type: "explorer" | "coder",
  status: SubagentInfo["status"],
): SubagentInfo {
  return {
    sessionId,
    name: type,
    type,
    status,
    promptPreview: "",
    description: type === "coder" ? "Implémentation" : "Analyse",
  };
}
