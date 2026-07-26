import type { ForecastNote, ForecastNotesAnalysis, NoteRange } from "./forecast-notes-types";

export function buildNoteRange(analysis: ForecastNotesAnalysis | null): NoteRange {
  const firstPrediction = analysis?.predictions[0]?.date;
  const lastPrediction = analysis?.predictions[analysis.predictions.length - 1]?.date;
  const start = dateMs(firstPrediction ?? analysis?.input_summary.start);
  const end = dateMs(lastPrediction ?? analysis?.input_summary.end);
  if (start == null || end == null || start >= end) {
    const now = Date.now();
    return { start: now - 1, end: now + 1 };
  }
  return { start, end };
}

export function notePosition(date: string, range: NoteRange): number {
  const value = dateMs(date);
  if (value == null) return 0;
  const ratio = (value - range.start) / (range.end - range.start);
  return Math.max(0, Math.min(100, ratio * 100));
}

export function defaultNoteDate(analysis: ForecastNotesAnalysis | null, note?: ForecastNote) {
  return note?.date ?? analysis?.predictions[0]?.date ?? analysis?.input_summary.end ?? today();
}

export function formatNoteDate(date: string, locale: string): string {
  const value = dateMs(date);
  if (value == null) return date;
  return new Intl.DateTimeFormat(locale, { day: "2-digit", month: "2-digit", year: "numeric" })
    .format(new Date(value));
}

export function formatNoteAxisDate(date: string, locale: string): string {
  const value = dateMs(date);
  if (value == null) return date;
  return new Intl.DateTimeFormat(locale, { day: "2-digit", month: "2-digit" })
    .format(new Date(value));
}

export function formatNoteRangeYear(range: NoteRange, locale: string): string {
  const start = new Date(range.start);
  const end = new Date(range.end);
  const formatter = new Intl.DateTimeFormat(locale, { year: "numeric" });
  const startYear = formatter.format(start);
  const endYear = formatter.format(end);
  return startYear === endYear ? startYear : `${startYear}-${endYear}`;
}

export function dateInputValue(date: string): string {
  return date.slice(0, 10);
}

export function midpointDate(range: NoteRange): string {
  return new Date(range.start + (range.end - range.start) / 2).toISOString();
}

export function appScopedPath(path: string): string {
  const normalized = path.replaceAll("\\", "/");
  const marker = "/cl-go-dash/";
  const index = normalized.toLowerCase().indexOf(marker);
  if (index === -1) return normalized;
  return `/Beaver/${normalized.slice(index + marker.length)}`;
}

function today(): string {
  return new Date().toISOString().slice(0, 10);
}

function dateMs(date?: string): number | null {
  if (!date) return null;
  const value = new Date(date).getTime();
  return Number.isNaN(value) ? null : value;
}
