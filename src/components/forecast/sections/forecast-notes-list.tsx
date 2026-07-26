import type { TFunction } from "i18next";
import type { ForecastNote } from "./forecast-notes-types";
import { formatNoteDate } from "./forecast-notes-utils";

interface ForecastNotesListProps {
  notes: ForecastNote[];
  selectedId: string | null;
  locale: string;
  t: TFunction;
  onSelect: (id: string) => void;
}

export function ForecastNotesList({
  notes,
  selectedId,
  locale,
  t,
  onSelect,
}: ForecastNotesListProps) {
  if (!notes.length) {
    return <div className="fcn-list-empty">{t("forecast.notes.empty")}</div>;
  }
  return (
    <div className="fcn-list">
      {notes.map((note) => (
        <button
          key={note.id}
          type="button"
          className={`fcn-note-row ${note.id === selectedId ? "is-selected" : ""}`}
          onClick={() => onSelect(note.id)}
        >
          <span className={`fcn-note-source is-${note.source}`}>
            {t(`forecast.notes.sources.${note.source}`)}
          </span>
          <span className="fcn-note-main">
            <span className="fcn-note-title">{note.title}</span>
            <span className="fcn-note-meta">
              {formatNoteDate(note.date, locale)} · {t(`forecast.notes.types.${note.note_type}`)}
            </span>
          </span>
        </button>
      ))}
    </div>
  );
}
