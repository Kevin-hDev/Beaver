import { describe, expect, it } from "vitest";
import { estimateAgentMessagesTokens, resolveSessionContext, textUnits } from "../agent-token-estimate";
import type { AgentMessage, AgentSession } from "@/types/agent";

function msg(content: string): AgentMessage {
  return {
    id: "m1",
    role: "user",
    content,
    files: [],
    timestamp: new Date().toISOString(),
    tokens: 0,
  };
}

describe("agent-token-estimate", () => {
  it("garde le ratio ASCII historique", () => {
    expect(estimateAgentMessagesTokens([msg("a".repeat(400))])).toBe(100);
  });

  it("compte les accents comme non ASCII", () => {
    expect(estimateAgentMessagesTokens([msg("éé")])).toBe(1);
  });

  it("compte CJK et hangul avec prudence", () => {
    expect(estimateAgentMessagesTokens([msg("你".repeat(1000))])).toBe(1250);
    expect(estimateAgentMessagesTokens([msg("こ".repeat(1000))])).toBe(1250);
    expect(estimateAgentMessagesTokens([msg("한".repeat(1000))])).toBe(1250);
  });

  it("compte les emoji comme larges", () => {
    expect(textUnits("🎉")).toBe(5);
    expect(estimateAgentMessagesTokens([msg("🎉")])).toBe(2);
  });

  it("ne recompte pas les copies UI du payload outil", () => {
    const content = "a".repeat(400);
    const message = msg("");
    message.role = "assistant";
    message.tool_activities = [{
      name: "write_file",
      summary: "/memory/preference.md",
      args: { path: "/memory/preference.md", content },
      result: "ok",
      content,
      domain: "memory",
    }];
    const expectedUnits = "write_file".length
      + JSON.stringify(message.tool_activities[0].args).length
      + 2;

    expect(estimateAgentMessagesTokens([message])).toBe(Math.ceil(expectedUnits / 4));
  });

  it("distingue le total complet du simple compteur de messages", () => {
    const session = {
      accumulated_tokens: 100,
      context_tokens: 900,
      messages: [msg("a".repeat(400))],
    } as AgentSession;

    expect(resolveSessionContext(session)).toEqual({
      sessionTokenCount: 900,
      hasContextUsageSnapshot: true,
      contextUsageVisible: false,
    });

    delete session.context_tokens;
    expect(resolveSessionContext(session)).toEqual({
      sessionTokenCount: 100,
      hasContextUsageSnapshot: false,
      contextUsageVisible: false,
    });
  });

  it("affiche le contexte d'une session seulement après une réponse assistant", () => {
    const session = {
      accumulated_tokens: 100,
      messages: [msg("question"), { ...msg("réponse"), role: "assistant" }],
    } as AgentSession;

    expect(resolveSessionContext(session).contextUsageVisible).toBe(true);
  });
});
