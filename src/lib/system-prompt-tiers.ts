import type { SystemPromptTier } from "@/types/system-prompts";

// Ces textes décrivent la règle ; seul Rust décide du format réellement utilisé.
export const SYSTEM_PROMPT_TIER_OPTIONS = [
  { id: "compact", range: "≤ 25B" },
  { id: "detailed", range: "> 25B" },
] as const satisfies ReadonlyArray<{ id: SystemPromptTier; range: string }>;
