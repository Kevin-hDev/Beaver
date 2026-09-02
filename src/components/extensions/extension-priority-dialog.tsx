import { useState } from "react";
import { useTranslation } from "react-i18next";
import { DialogPortal } from "@/components/ui/dialog-portal";
import { MAX_PROTECTED_PLUGINS } from "@/lib/extension-discovery";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionIcon } from "./extension-icon";
import { extensionDisplayName } from "./official-plugin-copy";
import "./extension-priority-dialog.css";

interface ExtensionPriorityDialogProps {
  records: ExtensionRecord[];
  selectedIds: string[];
  busy: boolean;
  onCancel: () => void;
  onSave: (ids: string[]) => Promise<void>;
}

export function ExtensionPriorityDialog({
  records,
  selectedIds,
  busy,
  onCancel,
  onSave,
}: ExtensionPriorityDialogProps) {
  const { t } = useTranslation();
  const [selected, setSelected] = useState(selectedIds);
  const eligible = records.filter((record) => record.enabled);
  const toggle = (id: string) => {
    setSelected((current) => {
      if (current.includes(id)) return current.filter((value) => value !== id);
      if (current.length >= MAX_PROTECTED_PLUGINS) return current;
      return [...current, id];
    });
  };

  /* Par le portail, comme toute couche flottante : cette fenêtre est rendue
     dans une carte de réglages, dont le fond flouté fait d'elle le repère de
     tout ce qu'elle contient. Sans ce détour, « posé sur la fenêtre » devenait
     « posé sur la carte », et la fenêtre s'ouvrait tronquée à ses bords. */
  return (
    <DialogPortal>
      <div
        className="wk-dialog-overlay"
        role="button"
        tabIndex={-1}
        aria-label={t("extensions.actions.cancel")}
        onClick={(event) => {
          if (event.target === event.currentTarget && !busy) onCancel();
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape" && !busy) onCancel();
        }}
      >
        <div
          className="wk-dialog extpd-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby="extpd-title"
        >
          <h3 id="extpd-title">{t("extensions.discovery.dialogTitle")}</h3>
          <p className="extpd-description">
            {t("extensions.discovery.dialogDescription", {
              count: selected.length,
              max: MAX_PROTECTED_PLUGINS,
            })}
          </p>
          <div className="extpd-list">
            {eligible.map((extension) => {
              const id = extension.manifest.id;
              const checked = selected.includes(id);
              return (
                <label className="extpd-row" key={id}>
                  <span className="extpd-icon">
                    <ExtensionIcon extension={extension} />
                  </span>
                  <span>{extensionDisplayName(t, extension)}</span>
                  <input
                    type="checkbox"
                    checked={checked}
                    disabled={busy || (!checked && selected.length >= MAX_PROTECTED_PLUGINS)}
                    onChange={() => toggle(id)}
                  />
                </label>
              );
            })}
            {eligible.length === 0 && (
              <span className="extpd-empty">
                {t("extensions.discovery.noEnabledPlugins")}
              </span>
            )}
          </div>
          <div className="wk-dialog-footer">
            <button
              type="button"
              className="btn btn-sm btn-secondary"
              disabled={busy}
              onClick={onCancel}
            >
              {t("extensions.actions.cancel")}
            </button>
            <button
              type="button"
              className="btn btn-sm btn-primary"
              disabled={busy}
              onClick={() => void onSave(selected)}
            >
              {t("extensions.discovery.validate")}
            </button>
          </div>
        </div>
      </div>
    </DialogPortal>
  );
}
