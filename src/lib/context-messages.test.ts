import { describe, expect, it } from "vitest";
import {
  isCloneSummaryContextMessage,
  isCompressionContextOnlyMessage,
  isCompressionSummaryMessage,
} from "./context-messages";
import type { AgentMessage } from "@/types/agent";

function msg(content: string, role: AgentMessage["role"] = "assistant"): AgentMessage {
  return {
    id: "m1",
    role,
    content,
    files: [],
    timestamp: new Date().toISOString(),
  };
}

describe("context messages", () => {
  it("detecte un resume de compression", () => {
    const message = msg("This session is being continued from a previous conversation.\n\nSummary");
    expect(isCompressionSummaryMessage(message)).toBe(true);
    expect(isCompressionContextOnlyMessage(message)).toBe(true);
  });

  it("detecte un resume de compression meme avec role user", () => {
    const message = msg(
      "This session is being continued from a previous conversation.\n\nSummary",
      "user",
    );
    expect(isCompressionSummaryMessage(message)).toBe(true);
    expect(isCompressionContextOnlyMessage(message)).toBe(true);
  });

  it("detecte le contexte fichiers conserve", () => {
    const message = msg("Recent file context preserved across compression:\n- read_file: app.ts");
    expect(isCompressionSummaryMessage(message)).toBe(false);
    expect(isCompressionContextOnlyMessage(message)).toBe(true);
  });

  it("detecte un resume cache de clone", () => {
    const message = msg("This cloned session includes hidden branch context:\n\nSummary", "user");
    expect(isCloneSummaryContextMessage(message)).toBe(true);
    expect(isCompressionContextOnlyMessage(message)).toBe(true);
  });

  it("detecte un rapport cache de sous-agent", () => {
    const message = msg("Subagent report context:\n<subagent id=\"s1\">Résumé</subagent>", "user");
    expect(isCompressionContextOnlyMessage(message)).toBe(true);
  });

  it("detecte un rappel cache de pilotage sous-agent", () => {
    const message = msg("Subagent orchestration context:\n<subagent_orchestration />", "user");
    expect(isCompressionContextOnlyMessage(message)).toBe(true);
  });

  it("ignore les messages normaux", () => {
    const message = msg("Voici la reponse visible pour l'utilisateur.");
    expect(isCompressionSummaryMessage(message)).toBe(false);
    expect(isCompressionContextOnlyMessage(message)).toBe(false);
  });
});
