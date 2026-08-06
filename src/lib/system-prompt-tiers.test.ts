import { describe, expect, it } from "vitest";
import { systemPromptTierForModel } from "./system-prompt-tiers";

describe("systemPromptTierForModel", () => {
  it.each([
    ["model:24b", "compact"],
    ["model:25b", "detailed"],
    ["model:24.5b", "compact"],
    ["model:25.5b", "detailed"],
    ["gemma4:e2b", "compact"],
    ["small-model", "compact"],
    ["unknown-model", "detailed"],
  ] as const)("classe %s dans le format %s", (model, expected) => {
    expect(systemPromptTierForModel(model)).toBe(expected);
  });
});
