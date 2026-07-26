import type { AvailableModel } from "@/hooks/available-model-types";

export type ReasoningMode =
  | "off"
  | "auto"
  | "low"
  | "medium"
  | "high"
  | "xhigh"
  | "max"
  | "ultra";

export interface ReasoningModeOption {
  mode: ReasoningMode;
  labelKey: string;
}

const LABELS: Record<ReasoningMode, string> = {
  off: "agentLocal.reasoningOff",
  auto: "agentLocal.reasoningAuto",
  low: "agentLocal.reasoningLow",
  medium: "agentLocal.reasoningMedium",
  high: "agentLocal.reasoningHigh",
  xhigh: "agentLocal.reasoningXhigh",
  max: "agentLocal.reasoningMax",
  ultra: "agentLocal.reasoningUltra",
};

function option(mode: ReasoningMode): ReasoningModeOption {
  return { mode, labelKey: LABELS[mode] };
}

function options(modes: ReasoningMode[]): ReasoningModeOption[] {
  return modes.map(option);
}

function modelName(model: AvailableModel | null): string {
  return model?.id.toLowerCase() ?? "";
}

function isGptOss(model: AvailableModel | null): boolean {
  return modelName(model).includes("gpt-oss");
}

function isGroqGptOssEffort(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return name.includes("gpt-oss-20b") || name.includes("gpt-oss-120b");
}

function isGroqQwenSwitchable(model: AvailableModel | null): boolean {
  return modelName(model).includes("qwen3-32b");
}

function isMistralNative(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return name.startsWith("magistral-small") || name.startsWith("magistral-medium");
}

function isMistralAdjustable(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return name === "mistral-small-latest" || name === "mistral-medium-3-5" || name === "mistral-medium-3.5";
}

function isMoonshotForced(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return (
    name.startsWith("kimi-k2.7-code")
    || name.includes("k2-thinking")
    || name.includes("thinking-preview")
    || name.startsWith("kimi-for-coding")
  );
}

function isMoonshotK3(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return name === "k3" || name.startsWith("kimi-k3");
}

function isGrokFixedReasoning(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return name === "grok-4.20-0309-reasoning" || name === "grok-build-0.1";
}

function isGpt56(model: AvailableModel | null): boolean {
  return modelName(model).includes("gpt-5.6");
}

function isGpt56CodexUltra(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return name === "gpt-5.6-sol" || name === "gpt-5.6-terra";
}

function isGpt56Codex(model: AvailableModel | null): boolean {
  return modelName(model).startsWith("gpt-5.6-");
}

function isGrok45(model: AvailableModel | null): boolean {
  return modelName(model).endsWith("grok-4.5");
}

function isGeminiForcedReasoning(model: AvailableModel | null): boolean {
  const name = modelName(model);
  return name.startsWith("gemini-3") || name.startsWith("gemini-2.5-pro");
}

function isZaiEffortReasoning(model: AvailableModel | null): boolean {
  return modelName(model).startsWith("glm-5.2");
}

export function reasoningModeOptions(model: AvailableModel | null): ReasoningModeOption[] {
  if (!model?.supports_thinking) return [];
  if (model.reasoning_modes?.length) return options(model.reasoning_modes);
  switch (model.provider_id) {
    case "codex-oauth":
      if (isGpt56CodexUltra(model)) {
        return options(["low", "medium", "high", "xhigh", "max", "ultra"]);
      }
      if (isGpt56Codex(model)) {
        return options(["low", "medium", "high", "xhigh", "max"]);
      }
      return options(["low", "medium", "high", "xhigh"]);
    case "ollama":
      return isGptOss(model)
        ? options(["low", "medium", "high"])
        : options(["off", "auto"]);
    case "openai":
      return isGpt56(model)
        ? options(["off", "low", "medium", "high", "xhigh", "max"])
        : options(["off", "low", "medium", "high", "xhigh"]);
    case "openrouter":
      if (isGpt56(model)) return options(["off", "low", "medium", "high", "xhigh", "max"]);
      if (isGrok45(model)) return options(["low", "medium", "high"]);
      return options(["off", "auto", "low", "medium", "high", "xhigh"]);
    case "google":
      return isGeminiForcedReasoning(model)
        ? options(["low", "medium", "high"])
        : options(["off", "low", "medium", "high"]);
    case "groq":
      if (isGroqGptOssEffort(model)) return options(["low", "medium", "high"]);
      if (isGroqQwenSwitchable(model)) return options(["off", "auto"]);
      return options(["auto"]);
    case "deepseek":
      return options(["off", "high", "xhigh"]);
    case "xai":
      if (isGrok45(model)) return options(["low", "medium", "high"]);
      if (isGrokFixedReasoning(model)) return options(["auto"]);
      return options(["off", "low", "medium", "high"]);
    case "mistral":
      if (isMistralNative(model)) return options(["auto"]);
      if (isMistralAdjustable(model)) return options(["off", "high"]);
      return [];
    case "moonshot":
    case "moonshot-oauth":
      if (isMoonshotK3(model)) return options(["low", "high", "max"]);
      return isMoonshotForced(model) ? options(["auto"]) : options(["off", "auto"]);
    case "zai":
      return isZaiEffortReasoning(model)
        ? options(["off", "auto", "low", "medium", "high", "xhigh"])
        : options(["off", "auto"]);
    default:
      return options(["off", "auto"]);
  }
}

export function normalizeReasoningMode(
  requested: string | null | undefined,
  options: ReasoningModeOption[],
  preferred?: ReasoningMode | null,
): ReasoningMode | null {
  if (options.length === 0) return null;
  if (requested && options.some((option) => option.mode === requested)) {
    return requested as ReasoningMode;
  }
  if (preferred && options.some((option) => option.mode === preferred)) return preferred;
  if (options.some((option) => option.mode === "medium")) return "medium";
  if (options.some((option) => option.mode === "auto")) return "auto";
  return options.find((option) => option.mode !== "off")?.mode ?? options[0].mode;
}
