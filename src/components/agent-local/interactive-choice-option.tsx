import { ChevronRight, PencilSimple } from "@/components/ui/icons";
import type { KeyboardEventHandler } from "react";
import type {
  AgentInteractiveOption,
  AgentInteractiveQuestion,
} from "@/types/agent";
import { InteractiveChoiceTooltip } from "./interactive-choice-tooltip";
import "./interactive-choice-option.css";

export const OTHER_VALUE = "other";

export function withOtherOption(
  question: AgentInteractiveQuestion | undefined,
  description: string,
): AgentInteractiveOption[] {
  return [
    ...(question?.options ?? []),
    {
      id: OTHER_VALUE,
      label: OTHER_VALUE,
      description,
      recommended: false,
    },
  ];
}

interface InteractiveChoiceOptionProps {
  option: AgentInteractiveOption;
  position: number;
  active: boolean;
  disabled: boolean;
  otherLabel: string;
  recommendedLabel: string;
  onHover: () => void;
  onChoose: () => void;
  onKeyDown: KeyboardEventHandler<HTMLButtonElement>;
}

export function InteractiveChoiceOption({
  option,
  position,
  active,
  disabled,
  otherLabel,
  recommendedLabel,
  onHover,
  onChoose,
  onKeyDown,
}: InteractiveChoiceOptionProps) {
  const isOther = option.id === OTHER_VALUE;
  const label = isOther ? otherLabel : option.label;

  return (
    <button
      type="button"
      className={`icp-option${active ? " icp-active" : ""}`}
      onMouseEnter={onHover}
      onClick={onChoose}
      onKeyDown={onKeyDown}
      disabled={disabled}
    >
      <span className="icp-option-marker" aria-hidden="true">
        {isOther
          ? <PencilSimple className="icp-option-marker-icon" />
          : position}
      </span>
      <InteractiveChoiceTooltip className="icp-option-label-host" fullText={label}>
        <span className="icp-option-label">{label}</span>
      </InteractiveChoiceTooltip>
      {option.recommended && (
        <InteractiveChoiceTooltip className="icp-recommended-host" fullText={recommendedLabel}>
          <span className="icp-recommended">{recommendedLabel}</span>
        </InteractiveChoiceTooltip>
      )}
      {option.description && (
        <InteractiveChoiceTooltip className="icp-option-description" fullText={option.description}>
          <span className="icp-option-description-text">{option.description}</span>
        </InteractiveChoiceTooltip>
      )}
      {active && <ChevronRight className="icp-arrow" aria-hidden="true" />}
    </button>
  );
}
