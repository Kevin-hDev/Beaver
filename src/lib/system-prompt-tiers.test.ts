import { describe, expect, it } from "vitest";
import { SYSTEM_PROMPT_TIER_OPTIONS } from "./system-prompt-tiers";

describe("SYSTEM_PROMPT_TIER_OPTIONS", () => {
  it("décrit la frontière décidée par le backend", () => {
    expect(SYSTEM_PROMPT_TIER_OPTIONS).toEqual([
      { id: "compact", range: "≤ 25B" },
      { id: "detailed", range: "> 25B" },
    ]);
  });
});
