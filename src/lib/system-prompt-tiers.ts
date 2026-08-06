import type { SystemPromptTier } from "@/types/system-prompts";

export const SYSTEM_PROMPT_TIER_OPTIONS = [
  { id: "compact", range: "< 25B" },
  { id: "detailed", range: "≥ 25B" },
] as const satisfies ReadonlyArray<{ id: SystemPromptTier; range: string }>;

export function systemPromptTierForModel(model: string): SystemPromptTier {
  for (const part of model.toLowerCase().split(/[:_-]/)) {
    if (!part.endsWith("b")) continue;
    const parameterCount = Number(part.slice(0, -1));
    if (Number.isFinite(parameterCount) && parameterCount >= 0) {
      return parameterCount < 25 ? "compact" : "detailed";
    }
  }

  return /(?:small|mini|tiny|nano|micro|e2b|e4b|lite)/i.test(model)
    ? "compact"
    : "detailed";
}
