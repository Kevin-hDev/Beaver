import { describe, expect, it, vi } from "vitest";
import { renderHook } from "@testing-library/react";

vi.mock("@/hooks/use-context-hidden-usage", () => ({
  useContextHiddenUsage: () => ({ includeThinking: true }),
}));

import { useContextUsage } from "@/hooks/use-context-usage";

describe("useContextUsage", () => {
  it("recalcule les messages après la fin d'une requête", () => {
    const identity = {
      requestId: "request", turnId: "turn", turn: 0, attempt: 1,
      providerId: "codex-oauth", model: "gpt-5.6-luna",
    };
    const stream = {
      completedSegments: [],
      currentContent: "",
      currentContentPhase: "final" as const,
      currentThinking: "",
      currentTools: [],
      contextUsageBuckets: null,
      contextUsageBaseSegments: 0,
      contextUsageIncludesReasoning: false,
      contextUsageVisible: true,
      contextUsageRecord: {
        activeRequestId: null,
        currentPreparation: {
          identity, contextLimit: 258_400,
          input: { tokens: 7_772, capacityTokens: 7_772, source: "heuristic" as const, coverage: "complete" as const },
          state: "completed" as const,
          breakdown: {
            messages: 2, systemTools: 4_385, mcpConnectors: 344,
            skills: 0, memory: 0, metaContext: 136, systemPrompt: 3_430,
            reasoningIncluded: false,
          },
          transientOverheadTokens: 0,
          updatedAt: "2026-09-12T00:47:21Z",
        },
        lastMeasurement: null,
        lastOutput: null,
      },
    };
    const messages = [
      {
        id: "user", role: "user" as const, content: "salut", files: [],
        timestamp: "2026-09-12T00:47:00Z",
      },
      {
        id: "assistant", role: "assistant" as const,
        content: "Salut ! Que puis-je faire pour toi ?", files: [],
        timestamp: "2026-09-12T00:47:21Z",
      },
    ];

    const { result } = renderHook(() => useContextUsage({
      sessionId: "session-test",
      model: "gpt-5.6-luna",
      provider: "codex-oauth",
      messages,
      stream,
      contextUsageIncludesReasoning: false,
    }));

    expect(result.current.items.find((item) => item.key === "messages")?.tokens).toBe(11);
  });

  it("respecte la métadonnée du modèle sans connaître son provider", () => {
    const message = {
      id: "message-test",
      role: "assistant" as const,
      content: "réponse",
      thinking: "raisonnement privé",
      files: [],
      timestamp: "2026-08-28T00:00:00Z",
    };
    const stream = {
      completedSegments: [],
      currentContent: "",
      currentContentPhase: "final" as const,
      currentThinking: "",
      currentTools: [],
      contextUsageBuckets: null,
      contextUsageBaseSegments: 0,
      contextUsageIncludesReasoning: false,
      contextUsageVisible: false,
      contextUsageRecord: {
        activeRequestId: null, currentPreparation: null,
        lastMeasurement: null, lastOutput: null,
      },
    };

    const { result } = renderHook(() => useContextUsage({
      sessionId: "session-test",
      model: "modele-fictif",
      provider: "provider-fictif",
      messages: [message],
      stream,
      contextUsageIncludesReasoning: false,
    }));

    expect(result.current.items.find((item) => item.key === "messages")?.tokens).toBe(2);
  });
});
