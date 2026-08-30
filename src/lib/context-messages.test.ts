import { describe, expect, it } from "vitest";
import {
  isCloneSummaryContextMessage,
  isCompressionContextOnlyMessage,
  isCompressionSummaryMessage,
} from "./context-messages";
import type { AgentMessage } from "@/types/agent";

function msg(
  content: string,
  role: AgentMessage["role"] = "assistant",
  messageKind?: "compression_checkpoint" | "compression_boundary",
): AgentMessage {
  return {
    id: "m1",
    role,
    content,
    message_kind: messageKind,
    files: [],
    timestamp: new Date().toISOString(),
  };
}

describe("context messages", () => {
  it("detecte un resume de compression", () => {
    const message = msg("Résumé au format libre", "user", "compression_checkpoint");
    expect(isCompressionSummaryMessage(message)).toBe(true);
    expect(isCompressionContextOnlyMessage(message)).toBe(true);
  });

  it("detecte un resume de compression meme avec role user", () => {
    const message = msg(
      "Résumé au format libre",
      "user",
      "compression_checkpoint",
    );
    expect(isCompressionSummaryMessage(message)).toBe(true);
    expect(isCompressionContextOnlyMessage(message)).toBe(true);
  });

  it("ne classe plus un ancien prefixe textuel sans kind", () => {
    const message = msg("Recent file context preserved across compression:\n- read_file: app.ts");
    expect(isCompressionSummaryMessage(message)).toBe(false);
    expect(isCompressionContextOnlyMessage(message)).toBe(false);
  });

  it("detecte la frontiere technique structuree", () => {
    const message = msg("Texte de frontière modifiable", "assistant", "compression_boundary");
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
