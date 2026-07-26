import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Pencil, Trash } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
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
  const [editing, setEditing] = useState(false);
  const [draftName, setDraftName] = useState(analysis.name);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const rootRef = useRef<HTMLDivElement | null>(null);

  const commitRename = useCallback(async () => {
    const trimmed = draftName.trim();
    if (!trimmed || trimmed === analysis.name) {
      setEditing(false);
      setDraftName(analysis.name);
      return;
    }
    await onRename(analysis.id, trimmed);
    setEditing(false);
  }, [analysis.id, analysis.name, draftName, onRename]);

  const confirmDeleteAnalysis = useCallback(async () => {
    await onDelete(analysis.id);
    setConfirmDelete(false);
  }, [analysis.id, onDelete]);

  useEffect(() => {
    if (!editing && !confirmDelete) return;

    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setEditing(false);
        setConfirmDelete(false);
        setDraftName(analysis.name);
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setEditing(false);
        setConfirmDelete(false);
        setDraftName(analysis.name);
      }
      if (event.key === "Enter") {
        event.preventDefault();
        if (editing) void commitRename();
        if (confirmDelete) void confirmDeleteAnalysis();
      }
    };

    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [editing, confirmDelete, analysis.name, commitRename, confirmDeleteAnalysis]);

  return (
    <div
      ref={rootRef}
      className="fch-card"
      role="button"
      tabIndex={0}
      onClick={() => {
        if (!editing) onLoad(analysis.id);
      }}
      onKeyDown={(event) => {
        if (!editing && (event.key === "Enter" || event.key === " ")) {
          event.preventDefault();
          onLoad(analysis.id);
        }
      }}
    >
      <div className="fch-card-main">
        <span className="fch-name-row">
          {editing ? (
            <input
              className="fch-rename-input"
              value={draftName}
              autoFocus
              onClick={(event) => event.stopPropagation()}
              onChange={(event) => setDraftName(event.target.value)}
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
        {confirmDelete && (
          <div className="fch-confirm-popover">
            <button
              type="button"
              onClick={(event) => {
                event.stopPropagation();
                void confirmDeleteAnalysis();
              }}
            >
              {t("forecast.history.validate")}
            </button>
            <button
              type="button"
              onClick={(event) => {
                event.stopPropagation();
                setConfirmDelete(false);
              }}
            >
              {t("forecast.history.cancel")}
            </button>
          </div>
        )}
        <Tooltip label={t("forecast.history.edit")}>
          <button
            type="button"
            className="icon-btn fch-icon-btn"
            onClick={(event) => {
              event.stopPropagation();
              setConfirmDelete(false);
              setDraftName(analysis.name);
              setEditing(true);
            }}
          >
            <Pencil size="var(--icon-15)" />
          </button>
        </Tooltip>
      <Tooltip label={t("forecast.history.delete")}>
        <button
          type="button"
          className="icon-btn fch-icon-btn fch-icon-btn-danger"
          onClick={(event) => {
            event.stopPropagation();
            setEditing(false);
            setConfirmDelete(true);
          }}
        >
          <Trash size="var(--icon-15)" />
        </button>
      </Tooltip>
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
