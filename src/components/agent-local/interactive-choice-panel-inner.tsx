import { useCallback, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Check, HelpCircle } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";
import type {
  AgentInteractiveAnswer,
  AgentInteractiveChoiceRequest,
} from "@/types/agent";
import {
  InteractiveChoiceOption,
  OTHER_VALUE,
  withOtherOption,
} from "./interactive-choice-option";
import { useInteractiveChoiceKeyboard } from "./use-interactive-choice-keyboard";
import "./interactive-choice-panel.css";

interface InteractiveChoicePanelInnerProps {
  request: AgentInteractiveChoiceRequest;
  onResolved?: () => void;
  onError?: () => void;
}

export function InteractiveChoicePanelInner({
  request,
  onResolved,
  onError,
}: InteractiveChoicePanelInnerProps) {
  const { t } = useTranslation();
  const [step, setStep] = useState(0);
  const [activeIndex, setActiveIndex] = useState(0);
  const [answers, setAnswers] = useState<AgentInteractiveAnswer[]>([]);
  const [otherText, setOtherText] = useState("");
  const [otherMode, setOtherMode] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  const question = request.questions[step];
  const options = useMemo(
    () => withOtherOption(question, t("interactiveChoice.other")),
    [question, t],
  );

  const submitAnswer = useCallback(async (answer: AgentInteractiveAnswer) => {
    if (submitting) return;
    const nextAnswers = [...answers.filter((item) => item.questionIndex !== step), answer];
    if (step + 1 < request.questions.length) {
      setAnswers(nextAnswers);
      setStep((value) => value + 1);
      setActiveIndex(0);
      setOtherText("");
      setOtherMode(false);
      return;
    }
    setSubmitting(true);
    try {
      await invoke("respond_to_interactive_choice", {
        sessionId: request.sessionId,
        id: request.id,
        answers: nextAnswers,
      });
      onResolved?.();
    } catch {
      setSubmitting(false);
      onError?.();
    }
  }, [answers, onError, onResolved, request, step, submitting]);

  const choose = useCallback((option: ReturnType<typeof withOtherOption>[number]) => {
    if (!question) return;
    if (option.id === OTHER_VALUE) {
      setOtherMode(true);
      return;
    }
    void submitAnswer({
      questionIndex: step,
      selectedIds: option.id ? [option.id] : [],
      selectedLabels: [option.label],
    });
  }, [question, step, submitAnswer]);

  const submitOther = useCallback(() => {
    const custom = otherText.trim();
    if (!custom) return;
    void submitAnswer({
      questionIndex: step,
      selectedIds: [OTHER_VALUE],
      selectedLabels: [OTHER_VALUE],
      customAnswer: custom,
    });
  }, [otherText, step, submitAnswer]);

  const cancel = useCallback(() => {
    if (submitting) return;
    setSubmitting(true);
    void invoke("dismiss_interactive_choice", {
      sessionId: request.sessionId,
      id: request.id,
    }).then(() => onResolved?.()).catch(() => {
      setSubmitting(false);
      onError?.();
    });
  }, [onError, onResolved, request, submitting]);

  const closeOther = useCallback(() => setOtherMode(false), []);
  const {
    panelRef,
    optionsRef,
    onChoiceKeyDown,
    onOtherKeyDown,
  } = useInteractiveChoiceKeyboard({
    options,
    activeIndex,
    setActiveIndex,
    choose,
    cancel,
    submitOther,
    closeOther,
  });

  if (!question) return null;

  return (
    <div
      className="icp-panel"
      role="group"
      aria-label={t("interactiveChoice.title")}
      tabIndex={-1}
      ref={panelRef}
    >
      <div className="icp-header">
        <HelpCircle className="icp-icon" aria-hidden="true" />
        <span className="icp-step">
          {t("interactiveChoice.step", { current: step + 1, total: request.questions.length })}
        </span>
        <span className="icp-title">{question.header}</span>
      </div>
      <div className="icp-question">{question.question}</div>
      <div className="icp-options" ref={optionsRef}>
        {options.map((option, index) => (
          <InteractiveChoiceOption
            key={`${option.id ?? option.label}-${index}`}
            option={option}
            position={index + 1}
            active={index === activeIndex}
            disabled={submitting}
            otherLabel={t("interactiveChoice.otherLabel")}
            recommendedLabel={t("interactiveChoice.recommended")}
            onHover={() => setActiveIndex(index)}
            onChoose={() => choose(option)}
            onKeyDown={onChoiceKeyDown}
          />
        ))}
      </div>
      {otherMode && (
        <div className="icp-other-row">
          <input
            className="icp-other-input"
            value={otherText}
            onChange={(event) => setOtherText(event.target.value)}
            placeholder={t("interactiveChoice.otherPlaceholder")}
            autoFocus
            onKeyDown={onOtherKeyDown}
          />
          <button
            className="icon-btn icp-submit"
            type="button"
            onClick={submitOther}
            onKeyDown={onOtherKeyDown}
            disabled={!otherText.trim()}
          >
            <Check className="icp-submit-icon" aria-hidden="true" />
          </button>
        </div>
      )}
    </div>
  );
}
