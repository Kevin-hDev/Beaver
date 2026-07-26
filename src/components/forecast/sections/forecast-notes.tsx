import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";
import { ForecastNotesTimeline } from "./forecast-notes-timeline";
import { ForecastNotesHeader } from "./forecast-notes-header";
import { ForecastNotesList } from "./forecast-notes-list";
import { ForecastNotesDetail } from "./forecast-notes-detail";
import { useForecastChartResize } from "../use-forecast-chart-resize";
import type {
  ForecastNote,
  ForecastNoteDraft,
  ForecastNotesAnalysis,
  ForecastNotesProps,
} from "./forecast-notes-types";
import { appScopedPath, buildNoteRange, defaultNoteDate } from "./forecast-notes-utils";
import "../forecast-sections.css";
import "./forecast-scenario-menu.css";
import "./forecast-notes.css";
import "./forecast-notes-detail.css";
import "./forecast-notes-markdown.css";

export function ForecastNotes({ analysisId }: ForecastNotesProps) {
  const { t, i18n } = useTranslation();
  const [analysis, setAnalysis] = useState<ForecastNotesAnalysis | null>(null);
  const [notes, setNotes] = useState<ForecastNote[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draft, setDraft] = useState<ForecastNoteDraft | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const runLatest = useLatestRequest();
  const timeline = useForecastChartResize({ defaultHeight: 91, minHeight: 70, maxWindowRatio: 0.35 });

  const selectedNote = useMemo(
    () => notes.find((note) => note.id === selectedId) ?? notes[0] ?? null,
    [notes, selectedId],
  );
  const range = useMemo(() => buildNoteRange(analysis), [analysis]);

  const load = useCallback(async () => {
    try {
      const loaded = await runLatest(() => Promise.all([
          invoke<ForecastNotesAnalysis>("get_forecast_analysis", { id: analysisId }),
          invoke<ForecastNote[]>("list_forecast_notes", { analysisId }),
        ]));
      if (loaded === undefined) return;
      const [nextAnalysis, nextNotes] = loaded;
      setAnalysis(nextAnalysis);
      setNotes(nextNotes);
      setSelectedId((current) => current ?? nextNotes[0]?.id ?? null);
      setError(null);
    } catch {
      setError(t("forecast.notes.loadFailed"));
    }
  }, [analysisId, runLatest, t]);

  useEffect(() => {
    const timer = window.setTimeout(() => void load(), 0);
    return () => window.clearTimeout(timer);
  }, [load]);

  useEffect(() => {
    const handleFocus = () => void load();
    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, [load]);

  useEffect(() => (
    listenForecastAnalysisEvents([FORECAST_ANALYSIS_UPDATED], (event) => {
      if (event.analysis_id === analysisId) void load();
    })
  ), [analysisId, load]);

  const startCreate = () => {
    setDraft({
      date: defaultNoteDate(analysis, selectedNote ?? undefined),
      title: "",
      note_type: "context",
      content: "",
    });
  };

  const startEdit = (note: ForecastNote) => {
    setDraft({
      id: note.id,
      date: note.date,
      title: note.title,
      note_type: note.note_type,
      content: note.content,
    });
  };

  const saveDraft = async () => {
    if (!draft || !draft.title.trim()) return;
    setSaving(true);
    try {
      const command = draft.id ? "update_forecast_note" : "create_forecast_note";
      const request = draft.id
        ? { analysis_id: analysisId, note_id: draft.id, ...draft }
        : { analysis_id: analysisId, ...draft };
      const saved = await invoke<ForecastNote>(command, { request });
      setDraft(null);
      setSelectedId(saved.id);
      await load();
    } catch {
      setError(t("forecast.notes.saveFailed"));
    } finally {
      setSaving(false);
    }
  };

  const deleteNote = async (note: ForecastNote) => {
    try {
      await invoke("delete_forecast_note", { analysisId, noteId: note.id });
      setSelectedId(null);
      await load();
    } catch {
      setError(t("forecast.notes.deleteFailed"));
    }
  };

  const openNote = async (note: ForecastNote) => {
    try {
      await invoke("open_forecast_note", { analysisId, noteId: note.id });
    } catch {
      setError(t("forecast.notes.openFailed"));
    }
  };

  if (error) return <div className="fc-error">{error}</div>;

  return (
    <div className="fcn-root">
      <ForecastNotesHeader
        analysis={analysis}
        noteCount={notes.length}
        onCreate={startCreate}
      />
      <div className="fcn-content">
        <ForecastNotesTimeline
          notes={notes}
          selectedId={selectedNote?.id ?? null}
          range={range}
          height={timeline.chartHeight}
          locale={i18n.language}
          t={t}
          onSelect={(id) => {
            setSelectedId(id);
            setDraft(null);
          }}
        />
        <div className="fcn-timeline-resize" onPointerDown={timeline.startResize} onDoubleClick={timeline.resetHeight} />
        {selectedNote?.file_path && (
          <div className="fcn-path-row" title={selectedNote.file_path}>
            <strong>{appScopedPath(selectedNote.file_path)}</strong>
          </div>
        )}
        <div className="fcn-workspace">
          <ForecastNotesList
            notes={notes}
            selectedId={selectedNote?.id ?? null}
            locale={i18n.language}
            t={t}
            onSelect={(id) => {
              setSelectedId(id);
              setDraft(null);
            }}
          />
          <ForecastNotesDetail
            note={selectedNote}
            draft={draft}
            locale={i18n.language}
            t={t}
            saving={saving}
            onDraftChange={setDraft}
            onEdit={startEdit}
            onCancel={() => setDraft(null)}
            onSave={() => void saveDraft()}
            onOpen={(note) => void openNote(note)}
            onDelete={(note) => void deleteNote(note)}
          />
        </div>
      </div>
    </div>
  );
}
