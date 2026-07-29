import { useCallback, useEffect, useMemo, useState } from "react";
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
import { PlanApprovalPanel } from "./plan-approval-panel";
import "./interactive-choice-panel.css";

interface InteractiveChoicePanelProps {
  request?: AgentInteractiveChoiceRequest;
  onResolved?: () => void;
}

export function InteractiveChoicePanel({ request, onResolved }: InteractiveChoicePanelProps) {
  if (!request) return null;
  if (request.kind === "plan_approval") {
    return <PlanApprovalPanel key={request.id} request={request} onResolved={onResolved} />;
  }
  return <InteractiveChoicePanelInner key={request.id} request={request} onResolved={onResolved} />;
}

function InteractiveChoicePanelInner({
  request,
  onResolved,
}: {
  request: AgentInteractiveChoiceRequest;
  onResolved?: () => void;
}) {
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
    }
  }, [answers, onResolved, request, step, submitting]);

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
    }).then(() => onResolved?.()).catch(() => setSubmitting(false));
  }, [onResolved, request, submitting]);

  useEffect(() => {
    if (!question) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "ArrowUp") {
        event.preventDefault();
        setActiveIndex((value) => (value - 1 + options.length) % options.length);
      } else if (event.key === "ArrowDown") {
        event.preventDefault();
        setActiveIndex((value) => (value + 1) % options.length);
      } else if (event.key === "Enter" && !event.shiftKey) {
        event.preventDefault();
        if (otherMode) submitOther();
        else {
          const option = options[activeIndex];
          if (option) choose(option);
        }
      } else if (event.key === "Escape") {
        event.preventDefault();
        if (otherMode) setOtherMode(false);
        else cancel();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [activeIndex, cancel, choose, options, otherMode, question, request, submitOther]);

  if (!question) return null;

  return (
    <div className="icp-panel" role="group" aria-label={t("interactiveChoice.title")}>
      <div className="icp-header">
        <HelpCircle className="icp-icon" aria-hidden="true" />
        <span className="icp-step">
          {t("interactiveChoice.step", { current: step + 1, total: request.questions.length })}
        </span>
        <span className="icp-title">{question.header}</span>
      </div>
      <div className="icp-question">{question.question}</div>
      <div className="icp-options">
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
          />
          <button className="icon-btn icp-submit" type="button" onClick={submitOther} disabled={!otherText.trim()}>
            <Check className="icp-submit-icon" aria-hidden="true" />
          </button>
        </div>
      )}
    </div>
  );
}
