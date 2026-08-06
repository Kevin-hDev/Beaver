import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { DialogPortal } from "@/components/ui/dialog-portal";
import type { PromptReplacementDestination } from "./use-system-prompt-replacement";
import "./system-prompt-warning-dialog.css";
import "./system-prompt-loss-dialog.css";

/* Durée d'affichage de la confirmation de copie, avant retour au libellé. */
const COPY_FEEDBACK_MS = 2000;

interface SystemPromptLossDialogProps {
  content: string;
  destination: PromptReplacementDestination;
  onCancel: () => void;
  onContinue: () => void;
}

export function SystemPromptLossDialog({
  content,
  destination,
  onCancel,
  onContinue,
}: SystemPromptLossDialogProps) {
  const { t } = useTranslation();
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">("idle");
  const titleId = useId();
  const descriptionId = useId();
  const cancelRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    const previousFocus = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    cancelRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      previousFocus?.focus();
    };
  }, [onCancel]);

  /* Le libellé revient à son état d'origine : figé sur « copié », un second clic
     ne renvoie plus rien à l'écran et on ne sait pas s'il a été pris en compte. */
  useEffect(() => {
    if (copyState !== "copied") return;
    const timer = setTimeout(() => setCopyState("idle"), COPY_FEEDBACK_MS);
    return () => clearTimeout(timer);
  }, [copyState]);

  const copyPrompt = async () => {
    try {
      await navigator.clipboard.writeText(content);
      setCopyState("copied");
    } catch {
      setCopyState("error");
    }
  };

  return (
    <DialogPortal>
      <div
        className="spp-warning-overlay"
        role="presentation"
        onMouseDown={(event) => {
          if (event.target === event.currentTarget) onCancel();
        }}
      >
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
              {t("settings.systemPrompt.loss.title")}
            </h3>
          </div>
          <p id={descriptionId} className="spp-warning-description">
            {t("settings.systemPrompt.loss.body", {
              destination: t(`settings.systemPrompt.sources.${destination}`),
            })}
          </p>
          {copyState === "error" && (
            <p className="spld-error" role="alert">
              {t("settings.systemPrompt.loss.copyError")}
            </p>
          )}
          <div className="spp-warning-actions spld-actions">
            <button
              className="btn btn-sm btn-secondary spld-copy"
              type="button"
              onClick={() => { void copyPrompt(); }}
            >
              {t(`settings.systemPrompt.loss.${copyState === "copied" ? "copied" : "copy"}`)}
            </button>
            <button
              ref={cancelRef}
              className="btn btn-sm btn-secondary"
              type="button"
              onClick={onCancel}
            >
              {t("settings.systemPrompt.cancel")}
            </button>
            <button className="btn btn-sm btn-destructive" type="button" onClick={onContinue}>
              {t("settings.systemPrompt.loss.continue")}
            </button>
          </div>
        </section>
      </div>
    </DialogPortal>
  );
}
