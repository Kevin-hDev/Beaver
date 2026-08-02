import { useEffect, useId, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useKeyboard } from "@/hooks/use-keyboard";
import "./directory-access-prompt.css";

export interface DirectoryAccessPromptProps {
  allowedPaths: string[];
  onCancel: () => void;
  onSettings: () => void;
}

export function DirectoryAccessPrompt({
  allowedPaths,
  onCancel,
  onSettings,
}: DirectoryAccessPromptProps) {
  const { t } = useTranslation();
  const cancelRef = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const descriptionId = useId();
  useKeyboard({ onEscape: onCancel });

  useEffect(() => {
    cancelRef.current?.focus();
  }, []);

  return (
    <div
      className="dap-root"
      role="alertdialog"
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
    >
      <div className="dap-copy">
        <strong id={titleId} className="dap-title">
          {t("directoryAccess.title")}
        </strong>
        <span id={descriptionId} className="dap-description">
          {t("directoryAccess.description")}
        </span>
        <span className="dap-paths">
          {allowedPaths.map((path) => (
            <span key={path} className="dap-path" title={path}>{path}</span>
          ))}
        </span>
        <span className="dap-help">{t("directoryAccess.help")}</span>
      </div>
      <div className="dap-actions">
        <button ref={cancelRef} type="button" className="btn btn-sm btn-secondary" onClick={onCancel}>
          {t("common.cancel")}
        </button>
        <button type="button" className="btn btn-sm btn-primary" onClick={onSettings}>
          {t("directoryAccess.settings")}
        </button>
      </div>
    </div>
  );
}
