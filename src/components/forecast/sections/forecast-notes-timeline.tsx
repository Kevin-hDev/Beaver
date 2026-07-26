import type { TFunction } from "i18next";
import { useLayoutEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import type { ForecastNote, NoteRange } from "./forecast-notes-types";
import {
  formatNoteAxisDate,
  formatNoteDate,
  formatNoteRangeYear,
  midpointDate,
  notePosition,
} from "./forecast-notes-utils";
import "./forecast-notes-timeline.css";

interface ForecastNotesTimelineProps {
  notes: ForecastNote[];
  selectedId: string | null;
  range: NoteRange;
  height: number;
  locale: string;
  t: TFunction;
  onSelect: (id: string) => void;
}

export function ForecastNotesTimeline({
  notes,
  selectedId,
  range,
  height,
  locale,
  t,
  onSelect,
}: ForecastNotesTimelineProps) {
  const rootRef = useRef<HTMLElement | null>(null);
  const [width, setWidth] = useState(0);
  const markers = useMemo(() => buildMarkers(notes, range, width), [notes, range, width]);

  useLayoutEffect(() => {
    const element = rootRef.current;
    if (!element) return;
    const syncWidth = () => setWidth(element.getBoundingClientRect().width);
    syncWidth();
    const observer = new ResizeObserver(syncWidth);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  return (
    <section ref={rootRef} className="fcn-timeline" aria-label={t("forecast.notes.timeline")}>
      <div className="fcn-timeline-year">{formatNoteRangeYear(range, locale)}</div>
      <div className="fcn-rail" style={{ height }}>
        <div className="fcn-rail-track">
          <span className="fcn-rail-line" />
          {notes.map((note, index) => (
            <button
              key={note.id}
              type="button"
              className={`fcn-dot is-${note.source} ${note.id === selectedId ? "is-selected" : ""}`}
              style={dotStyle(note, range, index)}
              title={`${note.title} · ${formatNoteDate(note.date, locale)}`}
              onClick={() => onSelect(note.id)}
            />
          ))}
        </div>
        {notes.length === 0 && (
          <span className="fcn-rail-empty">{t("forecast.notes.timelineEmpty")}</span>
        )}
      </div>
      <div className="fcn-axis">
        <div className="fcn-axis-track">
          {markers.map((marker) => (
            <span
              key={marker.date}
              className={`fcn-axis-mark is-${marker.edge}`}
              style={{ left: `${marker.position}%` }}
            >
              {formatNoteAxisDate(marker.date, locale)}
            </span>
          ))}
        </div>
      </div>
    </section>
  );
}

function buildMarkers(notes: ForecastNote[], range: NoteRange, width: number) {
  const values = [
    new Date(range.start).toISOString(),
    midpointDate(range),
    new Date(range.end).toISOString(),
    ...notes.map((note) => note.date),
  ];
  const unique = Array.from(new Set(values.map((date) => date.slice(0, 10))));
  return unique
    .map((date) => ({
      date,
      position: notePosition(date, range),
      edge: notePosition(date, range) <= 2 ? "start" : notePosition(date, range) >= 98 ? "end" : "middle",
    }))
    .sort((a, b) => a.position - b.position)
    .filter((marker, index, markers) => {
      if (index === 0 || index === markers.length - 1) return true;
      const previous = markers[index - 1];
      return marker.position - previous.position >= markerGap(width);
    });
}

function markerGap(width: number): number {
  if (!Number.isFinite(width) || width <= 0) return 16;
  if (width < 380) return 24;
  if (width < 560) return 17;
  return 11;
}

function dotStyle(note: ForecastNote, range: NoteRange, index: number): CSSProperties {
  return {
    left: `${notePosition(note.date, range)}%`,
    top: "50%",
    zIndex: index + 1,
  };
}
