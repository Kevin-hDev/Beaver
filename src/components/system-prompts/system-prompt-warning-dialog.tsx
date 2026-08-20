import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { DialogPortal } from "@/components/ui/dialog-portal";
import "./system-prompt-warning-dialog.css";

export type SystemPromptWarningKind = "global" | "ollama";

interface SystemPromptWarningDialogProps {
  kind: SystemPromptWarningKind;
  onCancel: () => void;
  onContinue: () => void;
}

export function shouldShowSystemPromptWarning(kind: SystemPromptWarningKind): boolean {
  try {
    return localStorage.getItem(storageKey(kind)) !== "1";
  } catch {
    return true;
  }
}

export function SystemPromptWarningDialog({
  kind,
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
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onCancel]);

  const handleContinue = () => {
    if (remember) {
      try {
        localStorage.setItem(storageKey(kind), "1");
      } catch {
        // L'avertissement sera simplement réaffiché si le stockage est indisponible.
      }
    }
    onContinue();
  };

  /* Par le portail, comme sa voisine : cette fenêtre est rendue dans une carte
     de réglages, dont le fond flouté fait d'elle le repère de tout ce qu'elle
     contient — « posé sur la fenêtre » y devient « posé sur la carte ». */
  return (
    <DialogPortal>
      <div className="spp-warning-overlay">
        <section
          className="spp-warning-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby={titleId}
          aria-describedby={descriptionId}
        >
          <div className="spp-warning-heading">
            <span className="spp-warning-icon" aria-hidden="true">!</span>
            <h3 id={titleId} className="spp-warning-title">
              {t("settings.systemPrompt.warning.title")}
            </h3>
          </div>
          <p id={descriptionId} className="spp-warning-description">
            {t(`settings.systemPrompt.warning.${kind}.body`)}
          </p>
          <label className="spp-warning-remember">
            <input
              type="checkbox"
              checked={remember}
              onChange={(event) => setRemember(event.target.checked)}
            />
            <span>{t("settings.systemPrompt.warning.remember")}</span>
          </label>
          <div className="spp-warning-actions">
            <button className="btn btn-sm btn-secondary" type="button" onClick={onCancel}>
              {t("settings.systemPrompt.cancel")}
            </button>
            <button
              ref={continueRef}
              className="btn btn-sm btn-primary"
              type="button"
              onClick={handleContinue}
            >
              {t("settings.systemPrompt.warning.continue")}
            </button>
          </div>
        </section>
      </div>
    </DialogPortal>
  );
}

function storageKey(kind: SystemPromptWarningKind): string {
  return `system-prompt-warning-${kind}-v1`;
}
