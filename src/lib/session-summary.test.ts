import { describe, expect, it } from "vitest";
import { collectFileOperations } from "./file-preview-utils";
import {
  childSubagents,
  summarizeLastRequestChanges,
  visibleTodoRuns,
} from "./session-summary";
import type { AgentMessage, AgentSession, AgentSessionMeta, ToolFileChangeRecord } from "@/types/agent";

const RUN_ID = "123e4567-e89b-42d3-a456-426614174000";

function change(path: string, additions: number, deletions: number): ToolFileChangeRecord {
  return { path, status: "modified", additions, deletions };
}

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

  it("compte les fichiers modifiés par le terminal et donne le total de la bulle", () => {
    const messages = [message("run", [
      {
        name: "edit_file",
        summary: "/repo/a.ts",
        old_text: "x",
        new_text: "y",
        file_changes: [change("/repo/a.ts", 1, 1)],
      },
      {
        name: "bash",
        summary: "sed -i s/y/z/ a.ts && cp a.ts b.ts",
        file_changes: [change("/repo/a.ts", 2, 0), change("/repo/b.ts", 4, 0)],
      },
    ])];
    const bubble = collectFileOperations(messages);

    expect(summarizeLastRequestChanges(messages)).toEqual({
      additions: bubble.reduce((total, operation) => total + operation.additions, 0),
      deletions: bubble.reduce((total, operation) => total + operation.deletions, 0),
      files: bubble.length,
    });
    expect(summarizeLastRequestChanges(messages)).toEqual({ additions: 7, deletions: 1, files: 2 });
  });

  it("additionne les changements d'un même fichier, dans la bulle comme dans le résumé", () => {
    const messages = [message("run", [
      { name: "edit_file", summary: "/repo/a.ts", file_changes: [change("/repo/a.ts", 1, 1)] },
      { name: "bash", summary: "echo >> a.ts", file_changes: [change("/repo/a.ts", 2, 0)] },
    ])];

    expect(collectFileOperations(messages).map((operation) => [operation.additions, operation.deletions]))
      .toEqual([[3, 1]]);
    expect(summarizeLastRequestChanges(messages)).toEqual({ additions: 3, deletions: 1, files: 1 });
  });

  it("compte une seule fois un fichier écrit en relatif puis en absolu dans une ancienne conversation", () => {
    const messages = [message("old", [
      { name: "write_file", summary: "src/a.ts", content: "a\nb" },
      { name: "edit_file", summary: "/repo/src/a.ts", file_changes: [change("/repo/src/a.ts", 1, 1)] },
    ])];
    const bubble = collectFileOperations(messages, { baseDir: "/repo" });

    expect(bubble).toHaveLength(1);
    expect(summarizeLastRequestChanges(messages, "/repo")).toEqual({ additions: 3, deletions: 1, files: 1 });
  });

  it("fait le total de toute la réponse, comme la bulle, quand elle tient en plusieurs messages", () => {
    const summary = summarizeLastRequestChanges([
      { ...message("step-1", [{ name: "write_file", summary: "/repo/a.ts", content: "a\nb" }]), stream_run_id: RUN_ID, stream_part: "final" },
      { ...message("step-2", [{ name: "write_file", summary: "/repo/b.ts", content: "c" }]), stream_run_id: RUN_ID, stream_part: "final" },
    ]);

    expect(summary).toEqual({ additions: 3, deletions: 0, files: 2 });
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
    fast_mode_enabled: false,
    message_count: 0,
    parent_session_id: parent,
    subagent_type: type,
  };
}
