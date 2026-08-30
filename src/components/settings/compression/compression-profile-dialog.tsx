import { useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { DialogPortal } from "@/components/ui/dialog-portal";
import { useDialogKeyboard } from "@/components/ui/use-dialog-keyboard";

interface CompressionProfileDialogProps {
  sourceName: string;
  existingNames: string[];
  onCancel: () => void;
  onCreate: (name: string) => Promise<boolean>;
}

const NAME_MAX = 48;

export function CompressionProfileDialog({
  sourceName,
  existingNames,
  onCancel,
  onCreate,
}: CompressionProfileDialogProps) {
  const { t } = useTranslation();
  const titleId = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const dialogRef = useRef<HTMLElement>(null);
  const submittingRef = useRef(false);
  const [name, setName] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const trimmed = name.trim();
  const visibleLength = [...trimmed].length;
  const duplicate = existingNames.some((item) => item.toLocaleLowerCase() === trimmed.toLocaleLowerCase());
  const valid = visibleLength > 0 && visibleLength <= NAME_MAX && !duplicate && !submitting;

  useDialogKeyboard({ rootRef: dialogRef, initialFocusRef: inputRef, onEscape: onCancel });

  const create = async () => {
    if (!valid || submittingRef.current) return;
    submittingRef.current = true;
    setSubmitting(true);
    if (await onCreate(trimmed)) onCancel();
    else {
      submittingRef.current = false;
      setSubmitting(false);
    }
  };

  return (
    <DialogPortal>
      <div className="cpd-overlay">
        <button
          type="button"
          className="cpd-backdrop-dismiss"
          tabIndex={-1}
          aria-label={t("settings.advanced.compressionCancel")}
          onClick={onCancel}
        />
        <section ref={dialogRef} className="cpd-dialog relief" role="dialog" aria-modal="true" aria-labelledby={titleId}>
          <h3 id={titleId}>{t("settings.advanced.compressionNewProfile")}</h3>
          <p>{t("settings.advanced.compressionCreateFrom", { name: sourceName })}</p>
          <input
            ref={inputRef}
            className="field field-wide"
            value={name}
            maxLength={NAME_MAX * 2}
            aria-label={t("settings.advanced.compressionProfileName")}
            onChange={(event) => setName(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                void create();
              }
            }}
          />
          <span className="cpd-count">
            {visibleLength >= 36 ? t("settings.advanced.compressionCharactersLeft", { count: NAME_MAX - visibleLength }) : ""}
          </span>
          <div className="cpd-actions">
            <button type="button" className="btn btn-sm btn-secondary" onClick={onCancel}>
              {t("settings.advanced.compressionCancel")}
            </button>
            <button type="button" className="btn btn-sm btn-primary" disabled={!valid} onClick={() => { void create(); }}>
              {t("settings.advanced.compressionCreate")}
            </button>
          </div>
        </section>
      </div>
    </DialogPortal>
  );
}
