import { useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { FileText, FolderOpen, ShieldWarning } from "@/components/ui/icons";
import "./extension-add-dialog.css";

interface ExtensionAddDialogProps {
  onAdd: (path: string) => Promise<boolean>;
  onClose: () => void;
}

export function ExtensionAddDialog({ onAdd, onClose }: ExtensionAddDialogProps) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState(false);

  const choose = async (directory: boolean) => {
    setFailed(false);
    let selected: string | string[] | null;
    try {
      selected = await open(directory
        ? { directory: true, multiple: false }
        : {
            directory: false,
            multiple: false,
            filters: [{
              name: t("extensions.add.supportedFiles"),
              extensions: [
                "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "mtsx", "ctsx", "json",
              ],
            }],
          });
    } catch {
      setFailed(true);
      return;
    }
    if (typeof selected !== "string") return;
    setBusy(true);
    try {
      const added = await onAdd(selected);
      setBusy(false);
      if (added) onClose();
      else setFailed(true);
    } catch {
      setBusy(false);
      setFailed(true);
    }
  };
  const close = () => { if (!busy) onClose(); };

  return (
    <div
      className="wk-dialog-overlay"
      role="button"
      tabIndex={-1}
      aria-label={t("extensions.actions.cancel")}
      onClick={(event) => {
        if (event.target === event.currentTarget) close();
      }}
      onKeyDown={(event) => { if (event.key === "Escape") close(); }}
    >
      <div
        className="wk-dialog exta-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="exta-title"
      >
        <h3 id="exta-title">{t("extensions.add.title")}</h3>
        <div className="exta-warning">
          <ShieldWarning size="var(--icon-xl)" weight="fill" />
          <p>{t("extensions.add.fullAccessWarning")}</p>
        </div>
        {failed && (
          <p className="exta-error" role="alert">{t("extensions.errors.operation")}</p>
        )}
        <div className="exta-options">
          <button type="button" className="exta-option" disabled={busy} onClick={() => void choose(false)}>
            <FileText size="var(--icon-lg)" />
            <span>
              <strong>{t("extensions.add.file")}</strong>
              <small>{t("extensions.add.fileDescription")}</small>
            </span>
          </button>
          <button type="button" className="exta-option" disabled={busy} onClick={() => void choose(true)}>
            <FolderOpen size="var(--icon-lg)" />
            <span>
              <strong>{t("extensions.add.folder")}</strong>
              <small>{t("extensions.add.folderDescription")}</small>
            </span>
          </button>
        </div>
        <div className="wk-dialog-footer">
          <button type="button" className="wk-btn-secondary" disabled={busy} onClick={close}>
            {t("extensions.actions.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
