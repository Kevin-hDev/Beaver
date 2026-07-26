import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { CloneSummaryRunIndicator } from "./clone-summary-run-button";
import type { CloneMode } from "@/types/agent";
import "./clone-session-dialog.css";

type CloneChoice = CloneMode | "summary_focus";

interface CloneSessionDialogProps {
  canSummarize: boolean;
  busy: boolean;
  error?: string | null;
  onCancel: () => void;
  onAbort: () => void;
  onSubmit: (mode: CloneMode, customFocus?: string) => void;
}

export function CloneSessionDialog({
  canSummarize,
  busy,
  error,
  onCancel,
  onAbort,
  onSubmit,
}: CloneSessionDialogProps) {
  const { t } = useTranslation();
  const [choice, setChoice] = useState<CloneChoice>(canSummarize ? "summary" : "cut");
  const [focus, setFocus] = useState("");

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key.startsWith("Esc")) onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  const submit = (nextChoice = choice) => {
    if (busy) return;
    if (nextChoice === "summary_focus") {
      onSubmit("summary", focus);
      return;
    }
    onSubmit(nextChoice);
  };

  return (
    <div
      className="wk-dialog-overlay"
      role="button"
      tabIndex={-1}
      aria-label={t("agentLocal.clone.close")}
      onClick={onCancel}
      onKeyDown={(event) => { if (event.key === "Escape") onCancel(); }}
    >
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div
        className={`wk-dialog csp-dialog ${busy ? "csp-dialog-busy" : ""}`}
        onClick={(event) => event.stopPropagation()}
        role="dialog"
      >
        <header className="wk-dialog-header">
          <span>{t("agentLocal.clone.title")}</span>
          <button type="button" className="wk-dialog-close" onClick={onCancel}>
            <X size="var(--icon-md)" />
          </button>
        </header>

        <div className={`wk-form csp-body ${busy ? "csp-body-busy" : ""}`}>
          {busy ? (
            <>
              <div className="csp-loading">
                <CloneSummaryRunIndicator className="csp-running-indicator" />
              </div>
              <footer className="wk-dialog-footer">
                <button type="button" className="wk-btn-secondary" onClick={onAbort}>
                  {t("agentLocal.cancel")}
                </button>
              </footer>
            </>
          ) : (
            <>
              <ChoiceButton active={choice === "cut"} onClick={() => setChoice("cut")}>
                {t("agentLocal.clone.cut")}
              </ChoiceButton>
              <ChoiceButton
                active={choice === "summary"}
                disabled={!canSummarize}
                onClick={() => setChoice("summary")}
              >
                {t("agentLocal.clone.summary")}
              </ChoiceButton>
              <ChoiceButton
                active={choice === "summary_focus"}
                disabled={!canSummarize}
                onClick={() => setChoice("summary_focus")}
              >
                {t("agentLocal.clone.summaryFocus")}
              </ChoiceButton>

              {choice === "summary_focus" && (
                <textarea
                  className="csp-focus"
                  value={focus}
                  placeholder={t("agentLocal.clone.focusPlaceholder")}
                  onChange={(event) => setFocus(event.target.value)}
                />
              )}

              {error && <div className="csp-error">{t("agentLocal.clone.summaryFailed")}</div>}

              <footer className="wk-dialog-footer">
                <button type="button" className="wk-btn-secondary" onClick={onCancel}>
                  {t("agentLocal.cancel")}
                </button>
                {error && (
                  <button type="button" className="wk-btn-secondary" onClick={() => submit("cut")}>
                    {t("agentLocal.clone.withoutSummary")}
                  </button>
                )}
                <button type="button" className="wk-btn-primary" disabled={busy} onClick={() => submit()}>
                  {error ? t("agentLocal.retry.button") : t("agentLocal.clone.create")}
                </button>
              </footer>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function ChoiceButton({
  active,
  disabled,
  onClick,
  children,
}: {
  active: boolean;
  disabled?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      className={`csp-choice ${active ? "csp-choice-active" : ""}`}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </button>
  );
}
