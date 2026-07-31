import { describe, expect, it, vi } from "vitest";
import { applyStreamEvent, createManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function makeState(overrides: Partial<ManagedStreamState> = {}): ManagedStreamState {
  return { ...createManagedStreamState([], 0), streamStartedAt: null, segmentStartedAt: null, ...overrides };
}

describe("toolCall", () => {
  it("ajoute un outil à currentTools", () => {
    const event = { event: "toolCall" as const, data: { name: "bash", arguments: { command: "ls" } } };
    const { state } = applyStreamEvent(makeState(), event);
    expect(state.currentTools).toHaveLength(1);
    expect(state.currentTools[0].name).toBe("bash");
    expect(state.contextUsageVisible).toBe(true);
  });

  it("ignore les tools internes", () => {
    let state = makeState();
    const names = ["todo_history", "todo_pause", "todo_resume", "todo_delete", "agent_diagnostics", "ask_user_choice", "planmode"];
    for (const name of names) {
      state = applyStreamEvent(state, { event: "toolCall", data: { name, arguments: {} } }).state;
    }
    expect(state.currentTools).toHaveLength(0);
    expect(state.contextUsageVisible).toBe(true);
  });
});

describe("toolResult", () => {
  it("assigne le résultat par index valide", () => {
    const currentTools = [{ name: "bash", args: {} }, { name: "grep", args: {} }];
    const state = makeState({ currentTools });
    const result = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "grep", toolCallIndex: 1, content: "trouvé", isError: false },
    });
    expect(result.state.currentTools[1].result).toBe("trouvé");
    expect(result.state.currentTools[0].result).toBeUndefined();
  });

  it("attache les fichiers touchés par bash au tool courant", () => {
    const currentTools = [{ name: "bash", args: { command: "touch a.md" } }];
    const state = makeState({ currentTools });
    const result = applyStreamEvent(state, {
      event: "toolResult",
      data: {
        name: "bash",
        toolCallIndex: 0,
        content: "ok",
        isError: false,
        affectedPaths: ["/repo/a.md"],
      },
    });

    expect(result.state.currentTools[0].affectedPaths).toEqual(["/repo/a.md"]);
  });

  it("attache le diff historique au tool courant", () => {
    const state = makeState({ currentTools: [{ name: "write_file", args: {} }] });
    const fileChanges = [{
      path: "/repo/a.md",
      status: "modified" as const,
      additions: 1,
      deletions: 1,
    }];
    const result = applyStreamEvent(state, {
      event: "toolResult",
      data: {
        name: "write_file",
        toolCallIndex: 0,
        content: "ok",
        isError: false,
        fileChanges,
      },
    });

    expect(result.state.currentTools[0].fileChanges).toEqual(fileChanges);
  });

  it("attache la ligne de modification structurée au tool courant", () => {
    const state = makeState({ currentTools: [{ name: "edit_file", args: {} }] });
    const result = applyStreamEvent(state, {
      event: "toolResult",
      data: {
        name: "edit_file",
        toolCallIndex: 0,
        content: "ok",
        isError: false,
        startLine: 12,
      },
    });

    expect(result.state.currentTools[0].startLine).toBe(12);
  });

  it("attache le nom lisible d'un skill sans modifier ses arguments", () => {
    const args = { skill_id: "local:skill:0123456789abcdef" };
    const state = makeState({ currentTools: [{ name: "load_skill", args }] });
    const result = applyStreamEvent(state, {
      event: "toolResult",
      data: {
        name: "load_skill",
        toolCallIndex: 0,
        content: "Skill loaded",
        isError: false,
        displaySummary: "context7-docs",
      },
    });

    expect(result.state.currentTools[0].displaySummary).toBe("context7-docs");
    expect(result.state.currentTools[0].args).toEqual(args);
  });

  it("ignore les résultats des tools internes", () => {
    const state = makeState({ currentTools: [{ name: "bash", args: {} }] });
    const result = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "todo_history", toolCallIndex: 0, content: "hidden", isError: false },
    });
    expect(result.state.currentTools[0].result).toBeUndefined();
  });

  it("vide le choix interactif quand ask_user_choice retourne un résultat", () => {
    const state = makeState({ interactiveChoice: {
      sessionId: "session-1",
      id: "choice-1",
      currentIndex: 0,
      total: 1,
      questions: [{ header: "Plan", question: "Choisir ?", options: [] }],
    } });
    const result = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "ask_user_choice", toolCallIndex: 0, content: "ok", isError: false },
    });
    expect(result.state.interactiveChoice).toBeUndefined();
  });

  it("vide le choix interactif quand planmode retourne sa validation backend", () => {
    const state = makeState({ interactiveChoice: {
      sessionId: "session-1",
      id: "choice-1",
      currentIndex: 0,
      total: 1,
      questions: [{ header: "Plan", question: "Valider ?", options: [] }],
    } });
    const result = applyStreamEvent(state, {
      event: "toolResult",
      data: { name: "planmode", toolCallIndex: 0, content: "approved", isError: false },
    });
    expect(result.state.interactiveChoice).toBeUndefined();
  });
});
