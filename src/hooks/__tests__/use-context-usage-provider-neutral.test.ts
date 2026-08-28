import { describe, expect, it, vi } from "vitest";
import { renderHook } from "@testing-library/react";

vi.mock("@/hooks/use-context-hidden-usage", () => ({
  useContextHiddenUsage: () => ({ includeThinking: true }),
}));

import { useContextUsage } from "@/hooks/use-context-usage";

describe("useContextUsage", () => {
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
