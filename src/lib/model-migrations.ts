import type { ReasoningMode } from "./reasoning-modes";

interface LegacyModelRule {
  model: string;
  defaultMode: ReasoningMode | null;
}

export interface LegacyModelReplacement {
  model: string;
  reasoningMode: ReasoningMode | null;
}

const LEGACY_XAI_MODELS: Readonly<Record<string, LegacyModelRule>> = {
  "grok-4.20-reasoning": {
    model: "grok-4.20-0309-reasoning",
    defaultMode: "auto",
  },
  "grok-4.20-non-reasoning": {
    model: "grok-4.20-0309-non-reasoning",
    defaultMode: null,
  },
  "grok-code-fast": { model: "grok-build-0.1", defaultMode: "auto" },
  "grok-code-fast-1": { model: "grok-build-0.1", defaultMode: "auto" },
  "grok-code-fast-1-0825": { model: "grok-build-0.1", defaultMode: "auto" },
  "grok-4-1-fast-reasoning": { model: "grok-4.3", defaultMode: "low" },
  "grok-4-fast-reasoning": { model: "grok-4.3", defaultMode: "low" },
  "grok-4-0709": { model: "grok-4.3", defaultMode: "low" },
  "grok-4": { model: "grok-4.3", defaultMode: "low" },
  "grok-3-mini": { model: "grok-4.3", defaultMode: "low" },
  "grok-4-1-fast-non-reasoning": { model: "grok-4.3", defaultMode: "off" },
  "grok-4-fast-non-reasoning": { model: "grok-4.3", defaultMode: "off" },
  "grok-3": { model: "grok-4.3", defaultMode: "off" },
  "grok-3-fast": { model: "grok-4.3", defaultMode: "off" },
  "grok-2-vision": { model: "grok-4.3", defaultMode: "off" },
};

export function legacyXaiReplacement(
  provider: string,
  model: string,
  currentMode: string | null | undefined,
  supportedModes: readonly ReasoningMode[],
): LegacyModelReplacement | null {
  if (provider !== "xai") return null;
  const rule = LEGACY_XAI_MODELS[model];
  if (!rule) return null;

  const preservedMode = supportedModes.find((mode) => mode === currentMode);
  return {
    model: rule.model,
    reasoningMode: preservedMode ?? rule.defaultMode,
  };
}
