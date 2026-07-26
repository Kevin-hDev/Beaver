const MAX_PREVIEW_ROWS = 200;
const MAX_PREVIEW_COLUMNS = 32;
const MAX_CELL_CHARS = 120;

export interface WorkbenchInputData {
  columns?: string[];
  date_column?: string | null;
  rows?: Record<string, unknown>[];
  series_column?: string | null;
  history?: { date: string; value: number; series_id?: string | null }[];
}

export function buildDataPreview(input: WorkbenchInputData, targetColumn: string) {
  const sourceRows = input.rows?.length
    ? input.rows
    : (input.history ?? []).map((point) => ({
        [input.date_column || "date"]: point.date,
        [targetColumn]: point.value,
        ...(point.series_id == null || !input.series_column
          ? {}
          : { [input.series_column]: point.series_id }),
      }));
  const discovered = sourceRows[0] ? Object.keys(sourceRows[0]) : [];
  const columns = (input.columns?.length ? input.columns : discovered)
    .filter((column) => column.trim().length > 0)
    .slice(0, MAX_PREVIEW_COLUMNS);
  return {
    columns,
    rows: sourceRows.slice(0, MAX_PREVIEW_ROWS),
    totalRows: sourceRows.length,
    truncated: sourceRows.length > MAX_PREVIEW_ROWS,
  };
}

export function formatDataCell(value: unknown): string {
  if (value === null || value === undefined || value === "") return "—";
  let text: string;
  if (typeof value === "string") text = value;
  else if (typeof value === "number" || typeof value === "boolean") text = String(value);
  else {
    try {
      text = JSON.stringify(value);
    } catch {
      return "—";
    }
  }
  const clean = Array.from(text)
    .map((character) => {
      const code = character.codePointAt(0) ?? 0;
      return code < 32 || code === 127 ? " " : character;
    })
    .join("")
    .trim();
  const chars = Array.from(clean);
  return chars.length <= MAX_CELL_CHARS
    ? clean || "—"
    : `${chars.slice(0, MAX_CELL_CHARS - 1).join("")}…`;
}
