import { useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import {
  DownloadSimple,
  FileText,
  FolderOpen,
  GitBranch,
  ShieldWarning,
} from "@/components/ui/icons";
import type { ExtensionInstallSource } from "@/lib/extension-install";
import { cn } from "@/lib/utils";
import { ExtensionSourceForm } from "./extension-source-form";
import "./extension-add-dialog.css";

interface ExtensionAddDialogProps {
  onAdd: (path: string) => Promise<string | null>;
  onInstall: (source: ExtensionInstallSource, locator: string) => Promise<string | null>;
  onClose: () => void;
}

export function ExtensionAddDialog({ onAdd, onInstall, onClose }: ExtensionAddDialogProps) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [errorKey, setErrorKey] = useState<string | null>(null);
  const [source, setSource] = useState<ExtensionInstallSource | null>(null);
  const [locator, setLocator] = useState("");

  const choose = async (directory: boolean) => {
    setErrorKey(null);
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
      setErrorKey("extensions.errors.operation");
      return;
    }
    if (typeof selected !== "string") return;
    setBusy(true);
    try {
      const error = await onAdd(selected);
      setBusy(false);
      if (!error) onClose();
      else setErrorKey(error);
    } catch {
      setBusy(false);
      setErrorKey("extensions.errors.operation");
    }
  };
  const close = () => { if (!busy) onClose(); };
  const selectSource = (next: ExtensionInstallSource) => {
    if (busy) return;
    setSource(next);
    setLocator("");
    setErrorKey(null);
  };
  const submitSource = async () => {
    if (!source) return;
    const value = locator.trim();
    if (!value) {
      setErrorKey("extensions.errors.operation");
      return;
    }
    setBusy(true);
    setErrorKey(null);
    try {
      const error = await onInstall(source, value);
      setBusy(false);
      if (!error) onClose();
      else setErrorKey(error);
    } catch {
      setBusy(false);
      setErrorKey("extensions.errors.operation");
    }
  };

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
        {errorKey && (
          <p className="exta-error" role="alert">{t(errorKey)}</p>
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
          <button
            type="button"
            className={cn("exta-option", source === "git" && "exta-option-active")}
            disabled={busy}
            aria-pressed={source === "git"}
            onClick={() => selectSource("git")}
          >
            <GitBranch size="var(--icon-lg)" />
            <span>
              <strong>{t("extensions.add.git")}</strong>
              <small>{t("extensions.add.gitDescription")}</small>
            </span>
          </button>
          <button
            type="button"
            className={cn("exta-option", source === "npm" && "exta-option-active")}
            disabled={busy}
            aria-pressed={source === "npm"}
            onClick={() => selectSource("npm")}
          >
            <DownloadSimple size="var(--icon-lg)" />
            <span>
              <strong>{t("extensions.add.npm")}</strong>
              <small>{t("extensions.add.npmDescription")}</small>
            </span>
          </button>
        </div>
        {source && (
          <ExtensionSourceForm
            source={source}
            locator={locator}
            busy={busy}
            onLocatorChange={(value) => {
              setLocator(value);
              setErrorKey(null);
            }}
            onSubmit={() => void submitSource()}
          />
        )}
        <div className="wk-dialog-footer">
          <button type="button" className="wk-btn-secondary" disabled={busy} onClick={close}>
            {t("extensions.actions.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
