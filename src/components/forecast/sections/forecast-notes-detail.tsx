import type { TFunction } from "i18next";
import { useEffect, useMemo, useState } from "react";
import { ArrowSquareOut, Pencil, Trash } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { ForecastScenarioMenuSelect } from "./forecast-scenario-menu-select";
import { ForecastNotesMarkdown } from "./forecast-notes-markdown";
import type { ForecastNote, ForecastNoteDraft } from "./forecast-notes-types";
import { dateInputValue, formatNoteDate } from "./forecast-notes-utils";

interface ForecastNotesDetailProps {
  note: ForecastNote | null;
  draft: ForecastNoteDraft | null;
  locale: string;
  t: TFunction;
  saving: boolean;
  onDraftChange: (draft: ForecastNoteDraft) => void;
  onEdit: (note: ForecastNote) => void;
  onCancel: () => void;
  onSave: () => void;
  onOpen: (note: ForecastNote) => void;
  onDelete: (note: ForecastNote) => void;
}

export function ForecastNotesDetail(props: ForecastNotesDetailProps) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  useEffect(() => {
    if (!confirmDelete) return;
    const timer = window.setTimeout(() => setConfirmDelete(false), 5000);
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" || event.key.startsWith("Esc")) {
        event.preventDefault();
        setConfirmDelete(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.clearTimeout(timer);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [confirmDelete]);

  if (props.draft) return <ForecastNoteForm {...props} />;
  if (!props.note) return <div className="fcn-detail-empty">{props.t("forecast.notes.selectNote")}</div>;
  return (
    <section className="fcn-detail">
      <div className="fcn-detail-head">
        <div>
          <p className="fcn-detail-kicker">
            {formatNoteDate(props.note.date, props.locale)} · {props.t(`forecast.notes.types.${props.note.note_type}`)}
          </p>
          <h3>{props.note.title}</h3>
        </div>
        <div className="fcn-detail-actions">
          {confirmDelete && (
            <button
              className="fcn-confirm-delete"
              type="button"
              onClick={() => {
                props.onDelete(props.note as ForecastNote);
                setConfirmDelete(false);
              }}
            >
              {props.t("forecast.notes.confirmDelete")}
            </button>
          )}
          <Tooltip label={props.t("forecast.notes.open")}>
            <button type="button" className="icon-btn fcn-icon-btn" onClick={() => props.onOpen(props.note as ForecastNote)}>
              <ArrowSquareOut size="var(--icon-15)" />
            </button>
          </Tooltip>
          <Tooltip label={props.t("forecast.notes.edit")}>
            <button type="button" className="icon-btn fcn-icon-btn" onClick={() => props.onEdit(props.note as ForecastNote)}>
              <Pencil size="var(--icon-15)" />
            </button>
          </Tooltip>
          <Tooltip label={props.t("forecast.notes.delete")}>
            <button type="button" className="icon-btn fcn-icon-btn" onClick={() => setConfirmDelete((value) => !value)}>
              <Trash size="var(--icon-15)" />
            </button>
          </Tooltip>
        </div>
      </div>
      <ForecastNotesMarkdown content={props.note.content || props.note.title} />
    </section>
  );
}

function ForecastNoteForm({
  draft,
  t,
  saving,
  onDraftChange,
  onCancel,
  onSave,
}: ForecastNotesDetailProps) {
  const typeOptions = useMemo(
    () => ["context", "risk", "decision", "anomaly", "hypothesis"].map((value) => ({
      value,
      label: t(`forecast.notes.types.${value}`),
    })),
    [t],
  );
  if (!draft) return null;
  return (
    <section className="fcn-detail is-editing">
      <div className="fcn-form-grid">
        <input
          className="fcn-input"
          value={draft.title}
          placeholder={t("forecast.notes.titlePlaceholder")}
          onChange={(event) => onDraftChange({ ...draft, title: event.target.value })}
        />
        <input
          className="fcn-input"
          value={dateInputValue(draft.date)}
          placeholder={t("forecast.notes.datePlaceholder")}
          onChange={(event) => onDraftChange({ ...draft, date: event.target.value })}
        />
        <ForecastScenarioMenuSelect
          value={draft.note_type}
          options={typeOptions}
          className="fcn-type-menu"
          onChange={(value) => onDraftChange({ ...draft, note_type: value })}
        />
      </div>
      <textarea
        className="fcn-textarea"
        value={draft.content}
        placeholder={t("forecast.notes.contentPlaceholder")}
        onChange={(event) => onDraftChange({ ...draft, content: event.target.value })}
      />
      <div className="fcn-form-actions">
        <button type="button" className="fcn-primary-btn" disabled={saving} onClick={onSave}>
          {saving ? t("forecast.notes.saving") : t("forecast.notes.save")}
        </button>
        <button type="button" className="fcn-secondary-btn" onClick={onCancel}>
          {t("forecast.notes.cancel")}
        </button>
      </div>
    </section>
  );
}
