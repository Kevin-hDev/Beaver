import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import "./system-prompt-warning-dialog.css";

const DISMISSAL_KEY = "ollama-system-prompt-warning-dismissed-v1";

interface SystemPromptWarningDialogProps {
  onCancel: () => void;
  onContinue: () => void;
}

export function shouldShowSystemPromptWarning(): boolean {
  try {
    return localStorage.getItem(DISMISSAL_KEY) !== "1";
  } catch {
    return true;
  }
}

export function SystemPromptWarningDialog({
  onCancel,
  onContinue,
}: SystemPromptWarningDialogProps) {
  const { t } = useTranslation();
  const [remember, setRemember] = useState(false);
  const titleId = useId();
  const descriptionId = useId();
  const continueRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    continueRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onCancel]);

  const handleContinue = () => {
    if (remember) {
      try {
        localStorage.setItem(DISMISSAL_KEY, "1");
      } catch {
        // The warning will simply appear again if storage is unavailable.
      }
    }
    onContinue();
  };

  return (
    <div className="spw-overlay">
      <section
        className="spw-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descriptionId}
      >
        <div className="spw-heading">
          <span className="spw-icon" aria-hidden="true">
            !
          </span>
          <h3 id={titleId} className="spw-title">
            {t("ollama.systemPromptWarningTitle")}
          </h3>
        </div>

        <p id={descriptionId} className="spw-description">
          {t("ollama.systemPromptWarningBody")}
        </p>

        <label className="spw-remember">
          <input
            type="checkbox"
            checked={remember}
            onChange={(event) => setRemember(event.target.checked)}
          />
          <span>{t("ollama.systemPromptWarningRemember")}</span>
        </label>

        <div className="spw-actions">
          <button className="ollama-btn" type="button" onClick={onCancel}>
            {t("ollama.cancel")}
          </button>
          <button
            ref={continueRef}
            className="ollama-btn ollama-btn-primary"
            type="button"
            onClick={handleContinue}
          >
            {t("ollama.systemPromptWarningContinue")}
          </button>
        </div>
      </section>
    </div>
  );
}
