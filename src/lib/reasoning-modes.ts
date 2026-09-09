import type { AvailableModel } from "@/hooks/available-model-types";
import type { ReasoningModeId } from "@/types/model-reasoning-contract";

export type ReasoningMode = ReasoningModeId;

export interface ReasoningModeOption {
  mode: ReasoningMode;
  labelKey: string;
}

const LABELS: Record<ReasoningMode, string> = {
  off: "agentLocal.reasoningOff",
  auto: "agentLocal.reasoningAuto",
  minimal: "agentLocal.reasoningMinimal",
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

export function reasoningModeOptions(model: AvailableModel | null): ReasoningModeOption[] {
  if (!model?.supports_thinking) return [];
  const control = model.reasoning_contract?.control;
  if (control?.kind === "unknown" || control?.kind === "provider_default") return [];
  const modes = control?.kind === "toggle"
    ? (["off", "auto"] satisfies ReasoningMode[])
    : control?.kind === "efforts"
      ? control.efforts
      : (model.reasoning_modes ?? []);
  const hidesTechnicalAuto = model.provider_id === "anthropic"
    && modes.some((mode) => !["off", "auto"].includes(mode));
  return options(modes.filter((mode) => mode !== "auto" || !hidesTechnicalAuto));
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
