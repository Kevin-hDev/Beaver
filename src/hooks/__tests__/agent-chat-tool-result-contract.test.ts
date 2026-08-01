import { describe, expect, it, vi } from "vitest";
import {
  applyStreamEvent,
  createManagedStreamState,
  finishPartialStream,
} from "@/hooks/agent-chat-stream-callbacks";
import { expandToolActivities, toolsToRecords } from "@/hooks/agent-chat-utils";
import type { ManagedStreamState } from "@/hooks/agent-chat-stream-callbacks";
import type { ToolActivityRecord } from "@/types/agent";

vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function state(): ManagedStreamState {
  return createManagedStreamState([], 0);
}

describe("contrat des résultats d'outils", () => {
  it("ne décale pas les résultats quand un outil interne est masqué", () => {
    let current = state();
    current = toolCall(current, "todo_history", 0);
    current = toolCall(current, "bash", 1);
    current = toolCall(current, "grep", 2);

    current = toolResult(current, "grep", 2, "grep-result");
    current = toolResult(current, "todo_history", 0, "hidden-result");
    current = toolResult(current, "bash", 1, "bash-result");

    expect(current.currentTools.map((tool) => [tool.name, tool.result])).toEqual([
      ["bash", "bash-result"],
      ["grep", "grep-result"],
    ]);
  });

  it("utilise l'identifiant stable avant l'index ou le nom", () => {
    let current = state();
    current = toolCall(current, "grep", 3, "call-a");
    current = toolCall(current, "grep", 4, "call-b");
    current = applyStreamEvent(current, {
      event: "toolResult",
      data: {
        name: "grep",
        toolCallIndex: 3,
        toolCallId: "call-b",
        content: "second",
        isError: false,
      },
    }).state;

    expect(current.currentTools[0].result).toBeUndefined();
    expect(current.currentTools[1].result).toBe("second");
  });

  it("ignore un résultat stable dupliqué au lieu de le déplacer sur l'appel suivant", () => {
    let current = state();
    current = toolCall(current, "grep", 0, "call-a");
    current = toolCall(current, "grep", 1, "call-b");
    current = applyStreamEvent(current, {
      event: "toolResult",
      data: {
        name: "grep", toolCallIndex: 0, toolCallId: "call-a",
        content: "first", isError: false,
      },
    }).state;
    current = applyStreamEvent(current, {
      event: "toolResult",
      data: {
        name: "grep", toolCallIndex: 0, toolCallId: "call-a",
        content: "duplicate", isError: false,
      },
    }).state;

    expect(current.currentTools[0].result).toBe("first");
    expect(current.currentTools[1].result).toBeUndefined();
  });

  it("ignore un index stable inconnu au lieu de viser un outil du même nom", () => {
    let current = state();
    current = toolCall(current, "grep", 3);
    current = toolCall(current, "grep", 4);
    current = toolResult(current, "grep", 99, "wrong");

    expect(current.currentTools.every((tool) => tool.result === undefined)).toBe(true);
  });

  it("considère une sortie vide comme un succès terminé", () => {
    let current = toolCall(state(), "bash", 0);
    current = toolResult(current, "bash", 0, "");

    const finished = finishPartialStream(current).state;
    const lastMessage = finished.messages[finished.messages.length - 1];
    const saved: ToolActivityRecord | undefined = lastMessage?.tool_activities?.[0];
    expect(saved?.result).toBe("");
    expect(saved?.is_error).toBe(false);
    expect(saved?.result_meta?.status).toBe("success");
  });

  it("restaure une erreur structurée pour le modèle", () => {
    const record = toolsToRecords([{
      name: "bash",
      args: { command: "false" },
      result: "",
      isError: true,
      status: "error",
      error: {
        code: "shell_exit_nonzero",
        category: "execution",
        retryable: false,
      },
      truncated: true,
    }])[0];

    const restored = expandToolActivities([record], "");
    const envelope: unknown = JSON.parse(restored[1].content);
    expect(envelope).toMatchObject({
      status: "error",
      error: { code: "shell_exit_nonzero" },
      truncated: true,
    });
  });

  it("persiste une annulation locale avec des métadonnées cohérentes", () => {
    const current = toolCall(state(), "bash", 0);

    const finished = finishPartialStream(current);
    const record = finished.assistantMessage?.tool_activities?.[0];

    expect(record).toMatchObject({
      is_error: true,
      result_meta: {
        status: "cancelled",
        error: {
          code: "tool_cancelled",
          category: "cancelled",
          retryable: false,
        },
      },
    });
  });

  it("annule aussi un outil resté dans un segment déjà refermé", () => {
    const current = state();
    current.completedSegments = [{
      thinking: "",
      content: "",
      tools: [{ name: "bash", args: { command: "build" }, liveOutput: "running" }],
    }];

    const finished = finishPartialStream(current);
    const record = finished.assistantMessage?.segments?.[0].tools[0];

    expect(record?.result).toBe("running\n\nAnnulé.");
    expect(record?.result_meta?.status).toBe("cancelled");
  });

  it("marque un résultat inconnu après une interruption sans conseiller une relance aveugle", () => {
    const current = toolCall(state(), "write_file", 0);

    const finished = applyStreamEvent(current, {
      event: "error",
      data: { message: "provider_connection_failed" },
    });
    const record = finished.assistantMessage?.tool_activities?.[0];

    expect(record).toMatchObject({
      is_error: true,
      result_meta: {
        status: "error",
        error: {
          code: "tool_result_unavailable",
          category: "unavailable",
          retryable: false,
        },
      },
    });
  });

  it("signale un résultat manquant si le flux se termine normalement", () => {
    const current = toolCall(state(), "edit_file", 0);

    const finished = applyStreamEvent(current, {
      event: "done",
      data: {
        evalCount: 1,
        evalDurationNs: 1,
        finalTps: 1,
        promptTokens: 1,
        contextTokens: 1,
      },
    });
    const record = finished.assistantMessage?.tool_activities?.[0];

    expect(record?.result_meta?.error).toMatchObject({
      code: "tool_result_missing",
      category: "internal",
      retryable: false,
    });
  });
});

function toolCall(
  current: ManagedStreamState,
  name: string,
  toolCallIndex: number,
  toolCallId?: string,
): ManagedStreamState {
  return applyStreamEvent(current, {
    event: "toolCall",
    data: { name, arguments: {}, toolCallIndex, toolCallId },
  }).state;
}

function toolResult(
  current: ManagedStreamState,
  name: string,
  toolCallIndex: number,
  content: string,
): ManagedStreamState {
  return applyStreamEvent(current, {
    event: "toolResult",
    data: { name, toolCallIndex, content, isError: false, status: "success" },
  }).state;
}
