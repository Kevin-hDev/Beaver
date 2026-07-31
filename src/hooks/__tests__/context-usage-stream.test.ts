import { describe, expect, it } from "vitest";
import { resolvePreparedContextBuckets } from "../context-usage-stream";
import type { PreparedContextState } from "../context-usage-stream";
import type { ContextTokenBuckets } from "../context-usage-buckets";

function emptyContextBuckets(): ContextTokenBuckets {
  return {
    messages: 0,
    systemTools: 0,
    mcpConnectors: 0,
    skills: 0,
    memory: 0,
    metaContext: 0,
    systemPrompt: 0,
  };
}

function state(overrides: Partial<PreparedContextState> = {}): PreparedContextState {
  return {
    completedSegments: [],
    currentContent: "",
    currentThinking: "",
    currentTools: [],
    contextUsageBuckets: { ...emptyContextBuckets(), messages: 100 },
    contextUsageBaseSegments: 0,
    contextUsageIncludesReasoning: true,
    ...overrides,
  };
}

describe("context-usage-stream", () => {
  it("ajoute seulement les segments produits après le relevé préparé", () => {
    const base = { ...emptyContextBuckets(), messages: 100 };
    const usage = resolvePreparedContextBuckets(state({
      contextUsageBuckets: base,
      contextUsageBaseSegments: 1,
      completedSegments: [
        { content: "a".repeat(400), thinking: "", tools: [] },
        { content: "b".repeat(80), thinking: "", tools: [] },
      ],
      currentContent: "c".repeat(80),
    }));

    expect(usage?.messages).toBe(141);
    expect(base.messages).toBe(100);
  });

  it("n'ajoute pas le raisonnement que Codex ne rejoue pas", () => {
    const usage = resolvePreparedContextBuckets(state({
      contextUsageIncludesReasoning: false,
      currentThinking: "r".repeat(400),
    }));

    expect(usage?.messages).toBe(100);
  });
});
