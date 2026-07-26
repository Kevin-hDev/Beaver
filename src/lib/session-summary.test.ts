import { describe, expect, it } from "vitest";
import {
  addChangeSummaries,
  childSubagents,
  summarizeToolChange,
  summarizeLastRequestChanges,
  visibleTodoRuns,
} from "./session-summary";
import type { AgentMessage, AgentSession, AgentSessionMeta } from "@/types/agent";

function message(id: string, tools: AgentMessage["tool_activities"]): AgentMessage {
  return {
    id,
    role: "assistant",
    content: "",
    files: [],
    timestamp: "2026-07-01T12:00:00Z",
    tool_activities: tools,
  };
}

describe("session-summary", () => {
  it("compte les modifications de la dernière requête qui a modifié un fichier", () => {
    const summary = summarizeLastRequestChanges([
      message("old", [{ name: "write_file", summary: "old.ts", content: "a\nb" }]),
      message("new", [{ name: "edit_file", summary: "new.ts", old_text: "a\nb\nc", new_text: "x" }]),
    ]);

    expect(summary).toEqual({ additions: 1, deletions: 3, files: 1 });
  });

  it("reprend la dernière requête qui a modifié un fichier si le dernier message ne modifie rien", () => {
    const summary = summarizeLastRequestChanges([
      message("modified", [{ name: "write_file", summary: "old.ts", content: "a\nb" }]),
      message("chat", [{ name: "read_file", summary: "old.ts", result: "ok" }]),
    ]);

    expect(summary).toEqual({ additions: 2, deletions: 0, files: 1 });
  });

  it("additionne plusieurs modifications dans la même requête", () => {
    const summary = summarizeLastRequestChanges([
      message("multi", [
        { name: "write_file", summary: "new.ts", content: "a\nb\nc" },
        { name: "edit_file", summary: "old.ts", old_text: "x\ny", new_text: "z" },
      ]),
    ]);

    expect(summary).toEqual({ additions: 4, deletions: 2, files: 2 });
  });

  it("permet d'additionner les changements live sans remettre l'ancien total à zéro", () => {
    const first = summarizeToolChange({ name: "write_file", summary: "a.ts", content: "a" });
    const second = summarizeToolChange({ name: "edit_file", summary: "b.ts", old_text: "x\ny", new_text: "z" });

    expect(addChangeSummaries(first, second)).toEqual({ additions: 2, deletions: 2, files: 2 });
  });

  it("compte write_file comme additions sans suppressions", () => {
    const summary = summarizeLastRequestChanges([
      message("write", [{ name: "write_file", summary: "new.ts", content: "a\nb\nc" }]),
    ]);

    expect(summary).toEqual({ additions: 3, deletions: 0, files: 1 });
  });

  it("ignore les outils en erreur", () => {
    const summary = summarizeLastRequestChanges([
      message("failed", [{
        name: "edit_file",
        summary: "bad.ts",
        old_text: "a",
        new_text: "b",
        is_error: true,
      }]),
    ]);

    expect(summary).toEqual({ additions: 0, deletions: 0, files: 0 });
  });

  it("garde les todo runs actifs et en pause", () => {
    const session = {
      todo_runs: [
        { id: "a", title: "Active", status: "active", todos: [], created_at: "", updated_at: "" },
        { id: "p", title: "Paused", status: "paused", todos: [], created_at: "", updated_at: "" },
        { id: "c", title: "Completed", status: "completed", todos: [], created_at: "", updated_at: "" },
      ],
    } satisfies Pick<AgentSession, "todo_runs">;

    expect(visibleTodoRuns(session).map((run) => run.title)).toEqual(["Active", "Paused"]);
  });

  it("filtre les sous-agents par session parent", () => {
    const sessions = [
      meta("child-a", "parent", "explorer"),
      meta("other-child", "other", "coder"),
      meta("child-b", "parent", "coder"),
    ];

    expect(childSubagents("parent", sessions).map((agent) => agent.sessionId))
      .toEqual(["child-a", "child-b"]);
  });

  it("conserve les métadonnées visibles des sous-agents", () => {
    const sessions = [meta("child-a", "parent", "explorer")];
    sessions[0].subagent_description = "Analyse repo";
    sessions[0].subagent_color_key = "geminitor";
    sessions[0].subagent_summary = "Résumé final";

    expect(childSubagents("parent", sessions)[0]).toMatchObject({
      description: "Analyse repo",
      colorKey: "geminitor",
      summary: "Résumé final",
    });
  });
});

function meta(id: string, parent: string, type: "explorer" | "coder"): AgentSessionMeta {
  return {
    id,
    name: id,
    created_at: "2026-07-01T12:00:00Z",
    model: "gpt",
    provider: "openai",
    message_count: 0,
    parent_session_id: parent,
    subagent_type: type,
  };
}
