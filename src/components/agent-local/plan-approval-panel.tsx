import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PencilSimple } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";
import type { AgentInteractiveChoiceRequest } from "@/types/agent";
import { useFocusedPanel } from "./use-focused-panel";
import "./plan-approval-panel.css";

const IMPLEMENT_ID = "implement_plan";
const ADJUSTMENTS_ID = "other";
const MAX_ADJUSTMENT_LENGTH = 1500;

interface PlanApprovalPanelProps {
  request: AgentInteractiveChoiceRequest;
  onResolved?: () => void;
  onError?: () => void;
}

export function PlanApprovalPanel({
  request,
  onResolved,
  onError,
}: PlanApprovalPanelProps) {
  const { t } = useTranslation();
  const [adjustments, setAdjustments] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const respond = useCallback(async (
    selectedId: string,
    customAnswer?: string,
  ) => {
    if (submitting) return;
    setSubmitting(true);
    try {
      await invoke("respond_to_interactive_choice", {
        sessionId: request.sessionId,
        id: request.id,
        answers: [{
          questionIndex: 0,
          selectedIds: [selectedId],
          selectedLabels: [],
          ...(customAnswer ? { customAnswer } : {}),
        }],
      });
      onResolved?.();
    } catch {
      setSubmitting(false);
      onError?.();
    }
  }, [onError, onResolved, request.id, request.sessionId, submitting]);

  const implement = useCallback(() => {
    void respond(IMPLEMENT_ID);
  }, [respond]);

  const sendAdjustments = useCallback(() => {
    const text = adjustments.trim();
    if (!text) return;
    void respond(ADJUSTMENTS_ID, text);
  }, [adjustments, respond]);

  const dismiss = useCallback(async () => {
    if (submitting) return;
    setSubmitting(true);
    try {
      await invoke("dismiss_interactive_choice", {
        sessionId: request.sessionId,
        id: request.id,
      });
      onResolved?.();
    } catch {
      setSubmitting(false);
      onError?.();
    }
  }, [onError, onResolved, request.id, request.sessionId, submitting]);

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (event.key !== "Escape") return;
    event.preventDefault();
    event.stopPropagation();
    void dismiss();
  }, [dismiss]);
  const panelRef = useFocusedPanel(handleKeyDown);

  return (
    <div
      className="pap-panel"
      role="group"
      aria-label={t("interactiveChoice.planQuestion")}
      tabIndex={-1}
      ref={panelRef}
    >
      <p className="pap-question">{t("interactiveChoice.planQuestion")}</p>
      <button
        type="button"
        className="pap-implement"
        disabled={submitting}
        onClick={implement}
      >
        <span className="pap-number" aria-hidden="true">1</span>
        <span>{t("interactiveChoice.planImplement")}</span>
      </button>
      <div className="pap-adjust-row">
        <PencilSimple className="pap-pencil" aria-hidden="true" />
        <input
          className="pap-input"
          value={adjustments}
          maxLength={MAX_ADJUSTMENT_LENGTH}
          disabled={submitting}
          placeholder={t("interactiveChoice.planAdjustments")}
          aria-label={t("interactiveChoice.planAdjustments")}
          onChange={(event) => setAdjustments(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              event.preventDefault();
              sendAdjustments();
            }
          }}
        />
        <button
          type="button"
          className="pap-ignore"
          disabled={submitting}
          onClick={() => void dismiss()}
        >
          {t("interactiveChoice.ignore")}
          <kbd>ESC</kbd>
        </button>
        <button
          type="button"
          className="btn btn-sm btn-primary pap-send"
          disabled={submitting || !adjustments.trim()}
          onClick={sendAdjustments}
        >
          {t("interactiveChoice.send")}
        </button>
      </div>
    </div>
  );
}
