import { describe, expect, it } from "vitest";
import {
  estimateAgentMessagesTokens,
  resolveContextUsage,
  resolveSessionContext,
  textUnits,
} from "../agent-token-estimate";
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

  it("utilise le cumul de session avant la reconstruction des messages", () => {
    const session = {
      accumulated_tokens: 100,
      messages: [msg("a".repeat(400))],
    } as AgentSession;

    expect(resolveSessionContext(session)).toEqual({
      sessionTokenCount: 100,
      contextUsageRecord: {
        activeRequestId: null, currentPreparation: null,
        lastMeasurement: null, lastOutput: null,
      },
      contextUsageVisible: false,
    });

  });

  it("priorise la mesure et sa limite sur la préparation suivante", () => {
    const identity = (requestId: string) => ({
      requestId, turnId: `turn-${requestId}`, turn: 0, attempt: 1,
      providerId: "openai", model: requestId === "A" ? "gpt-a" : "gpt-b",
    });
    const resolved = resolveContextUsage({
      activeRequestId: null,
      currentPreparation: {
        identity: identity("B"), contextLimit: 100_000,
        input: { tokens: 80_000, capacityTokens: 80_000, source: "heuristic", coverage: "complete" },
        state: "completed", breakdown: null, transientOverheadTokens: 0, updatedAt: "2026-09-11T00:00:02Z",
      },
      lastMeasurement: {
        identity: identity("A"), contextLimit: 200_000,
        input: { tokens: 62_000, capacityTokens: 62_000, source: "provider", coverage: "complete" },
        updatedAt: "2026-09-11T00:00:01Z",
      },
      lastOutput: {
        identity: identity("B"),
        output: { tokens: 20, capacityTokens: 20, source: "provider", coverage: "complete" },
        updatedAt: "2026-09-11T00:00:02Z",
      },
    }, 45_000, 100_000);

    expect(resolved).toEqual({ used: 62_000, max: 200_000, output: 20 });
  });

  it("ignore un ancien historique de 900K quand la dernière requête mesurée vaut 31K", () => {
    const identity = {
      requestId: "A", turnId: "turn-A", turn: 0, attempt: 1,
      providerId: "codex-oauth", model: "gpt-5",
    };
    const resolved = resolveContextUsage({
      activeRequestId: null,
      currentPreparation: null,
      lastMeasurement: {
        identity, contextLimit: 258_400,
        input: { tokens: 31_000, capacityTokens: 31_000, source: "provider", coverage: "complete" },
        updatedAt: "2026-09-11T00:00:00Z",
      },
      lastOutput: null,
    }, 900_000, 258_400);

    expect(resolved).toEqual({ used: 31_000, max: 258_400, output: null });
  });

  it("reconstruit seulement un record vide et ne fabrique aucune limite historique", () => {
    const empty = { activeRequestId: null, currentPreparation: null, lastMeasurement: null, lastOutput: null };
    expect(resolveContextUsage(empty, 45_000, 200_000))
      .toEqual({ used: 45_000, max: 200_000, output: null });
    expect(resolveContextUsage(empty, 0, 200_000))
      .toEqual({ used: null, max: null, output: null });
  });

  it("conserve le précompte terminé du premier appel après reload", () => {
    const identity = { requestId: "B", turnId: "turn-B", turn: 0, attempt: 1, providerId: "openai", model: "gpt-5" };
    const resolved = resolveContextUsage({
      activeRequestId: null,
      currentPreparation: {
        identity, contextLimit: 200_000,
        input: { tokens: 6_000, capacityTokens: 6_000, source: "heuristic", coverage: "complete" },
        state: "completed", breakdown: null, transientOverheadTokens: 0, updatedAt: "2026-09-11T00:00:00Z",
      },
      lastMeasurement: null,
      lastOutput: null,
    });
    expect(resolved).toEqual({ used: 6_000, max: 200_000, output: null });
  });

  it("n'invente aucun pourcentage quand la mesure n'a pas de limite", () => {
    const identity = { requestId: "A", turnId: "turn-A", turn: 0, attempt: 1, providerId: "openai", model: "gpt-5" };
    const resolved = resolveContextUsage({
      activeRequestId: null, currentPreparation: null,
      lastMeasurement: {
        identity, contextLimit: null,
        input: { tokens: 62_000, capacityTokens: 64_000, source: "provider", coverage: "partial" },
        updatedAt: "2026-09-11T00:00:00Z",
      },
      lastOutput: null,
    });
    expect(resolved).toEqual({ used: 62_000, max: null, output: null });
  });

  it("conserve une vraie mesure nulle sans la remplacer par une reconstruction", () => {
    const identity = { requestId: "A", turnId: "turn-A", turn: 0, attempt: 1, providerId: "openai", model: "gpt-5" };
    const resolved = resolveContextUsage({
      activeRequestId: null, currentPreparation: null,
      lastMeasurement: {
        identity, contextLimit: 200_000,
        input: { tokens: 0, capacityTokens: 0, source: "provider", coverage: "complete" },
        updatedAt: "2026-09-11T00:00:00Z",
      },
      lastOutput: {
        identity,
        output: { tokens: 0, capacityTokens: 0, source: "provider", coverage: "complete" },
        updatedAt: "2026-09-11T00:00:00Z",
      },
    }, 45_000, 200_000);

    expect(resolved).toEqual({ used: 0, max: 200_000, output: 0 });
  });

  it("affiche le contexte d'une session seulement après une réponse assistant", () => {
    const session = {
      accumulated_tokens: 100,
      messages: [msg("question"), { ...msg("réponse"), role: "assistant" }],
    } as AgentSession;

    expect(resolveSessionContext(session).contextUsageVisible).toBe(true);
  });
});
