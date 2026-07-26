import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";

interface SwitchModelDialogProps {
  fromModel: string;
  toModel: string;
  onNewSession: (remember: boolean) => void;
  onContinue: (remember: boolean) => void;
  onCancel: () => void;
}

/**
 * Popup qui demande à l'utilisateur ce qu'il veut faire quand il change
 * de modèle en cours de conversation.
 * - Nouvelle session : crée une nouvelle conversation avec le nouveau modèle
 * - Continuer : garde les messages, change juste le modèle utilisé pour la suite
 * - Mémoriser : la préférence reste valable tant que la session est ouverte
 */
export function SwitchModelDialog({
  fromModel,
  toModel,
  onNewSession,
  onContinue,
  onCancel,
}: SwitchModelDialogProps) {
  const { t } = useTranslation();
  const [remember, setRemember] = useState(false);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        onCancel();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div className="wk-dialog-overlay" role="button" tabIndex={-1} aria-label="Close dialog" onClick={onCancel} onKeyDown={(e) => { if (e.key === "Escape") onCancel(); }}>
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div
        className="wk-dialog"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        style={{ maxWidth: 480 }}
      >
        <header className="wk-dialog-header">
          <span>{t("switchModel.title")}</span>
          <button type="button" className="wk-dialog-close" onClick={onCancel}>
            <X size="var(--icon-md)" />
          </button>
        </header>

        <div className="wk-form">
          <p style={{ margin: 0, fontSize: "var(--text-sm)", color: "var(--ink)" }}>
            {t("switchModel.description", { from: fromModel, to: toModel })}
          </p>

          <label
            style={{
              display: "flex",
              alignItems: "center",
              gap: 8,
              fontSize: "var(--text-sm)",
              color: "var(--ink-muted)",
              cursor: "pointer",
              marginTop: 4,
            }}
          >
            <input
              type="checkbox"
              checked={remember}
              onChange={(e) => setRemember(e.target.checked)}
              style={{ accentColor: "var(--pulse)" }}
            />
            {t("switchModel.remember")}
          </label>

          <footer className="wk-dialog-footer">
            <button
              type="button"
              className="wk-btn-secondary"
              onClick={() => onNewSession(remember)}
            >
              {t("switchModel.newSession")}
            </button>
            <button
              type="button"
              className="wk-btn-primary"
              onClick={() => onContinue(remember)}
            >
              {t("switchModel.continue")}
            </button>
          </footer>
        </div>
      </div>
    </div>
  );
}
