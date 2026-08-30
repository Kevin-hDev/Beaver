import { useRef } from "react";
import { useTranslation } from "react-i18next";
import {
  EditableRowActions,
  useEditableRowActions,
} from "@/components/ui/editable-row-actions";
import type { AnalysisMeta } from "./forecast-history";

interface ForecastHistoryRowProps {
  analysis: AnalysisMeta;
  onLoad: (id: string) => void;
  onRename: (id: string, name: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

export function ForecastHistoryRow({
  analysis,
  onLoad,
  onRename,
  onDelete,
}: ForecastHistoryRowProps) {
  const { t, i18n } = useTranslation();
  const rootRef = useRef<HTMLDivElement | null>(null);
  const actions = useEditableRowActions({
    rootRef,
    value: analysis.name,
    onRename: (name) => onRename(analysis.id, name),
    onDelete: () => onDelete(analysis.id),
  });

  return (
    <div
      ref={rootRef}
      className="fch-card"
      role="button"
      tabIndex={0}
      onClick={() => {
        if (!actions.editing) onLoad(analysis.id);
      }}
      onKeyDown={(event) => {
        if (!actions.editing && (event.key === "Enter" || event.key === " ")) {
          event.preventDefault();
          onLoad(analysis.id);
        }
      }}
    >
      <div className="fch-card-main">
        <span className="fch-name-row">
          {actions.editing ? (
            <input
              className="field fch-rename-input"
              value={actions.draft}
              autoFocus
              onClick={(event) => event.stopPropagation()}
              onChange={(event) => actions.setDraft(event.target.value)}
            />
          ) : (
            <span className="fch-name">{analysis.name}</span>
          )}
          {analysis.scenarios_count > 0 && <span className="fch-scenario-dot" />}
        </span>
        <span className="fch-meta">
          {analysis.model} · {t("forecast.history.points", { count: analysis.points })} ·{" "}
          {t("forecast.history.horizonShort", { count: analysis.horizon })}
          {analysis.mape != null && ` · ${t("forecast.history.mapeShort", { value: analysis.mape.toFixed(1) })}`}
        </span>
        <span className="fch-date">{formatTimestamp(analysis.created_at, i18n.language)}</span>
      </div>
      <div className="fch-actions">
        <EditableRowActions
          controller={actions}
          confirmationPlacement="side"
          renameLabel={t("forecast.history.edit")}
          deleteLabel={t("forecast.history.delete")}
          confirmLabel={t("forecast.history.validate")}
          cancelLabel={t("forecast.history.cancel")}
        />
      </div>
    </div>
  );
}

function formatTimestamp(value: string, language: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(language, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}
