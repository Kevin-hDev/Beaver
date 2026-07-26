import { invoke } from "@tauri-apps/api/core";
import {
  MAX_FORECAST_CELL_CHARS,
  MAX_FORECAST_COLUMN_CHARS,
  MAX_FORECAST_INLINE_DATA_BYTES,
  MAX_FORECAST_INPUT_COLUMNS,
  MAX_FORECAST_INPUT_ROWS,
  MAX_FORECAST_PREVIEW_CHARS,
} from "./forecast-limits";

export type ForecastDraftRow = Record<string, unknown>;

export interface ForecastDraftData {
  sourceName: string;
  columns: string[];
  rowCount: number;
  dataJson: string;
  rows: ForecastDraftRow[];
}

interface SpreadsheetPreview {
  headers: string[];
  rows: unknown[][];
  total_rows: number;
  truncated?: boolean;
}

export async function loadForecastDraftFromFile(path: string): Promise<ForecastDraftData> {
  const raw = await invoke<string>("read_selected_spreadsheet_preview", {
    path,
    maxRows: MAX_FORECAST_INPUT_ROWS,
  });
  if (raw.length > MAX_FORECAST_PREVIEW_CHARS) throw new Error("forecast_data_too_large");
  const preview = JSON.parse(raw) as SpreadsheetPreview;
  return buildForecastDraftData(preview, basename(path));
}

export function buildForecastDraftData(
  preview: SpreadsheetPreview,
  sourceName: string,
): ForecastDraftData {
  if (
    preview.truncated
    || !Number.isSafeInteger(preview.total_rows)
    || preview.total_rows < 0
    || preview.total_rows > MAX_FORECAST_INPUT_ROWS
    || preview.rows.length > MAX_FORECAST_INPUT_ROWS
  ) {
    throw new Error("forecast_data_too_large");
  }
  const columns = mapHeaders(preview.headers);
  const records = preview.rows.slice(0, MAX_FORECAST_INPUT_ROWS).map((row) => {
    if (!Array.isArray(row) || row.length > MAX_FORECAST_INPUT_COLUMNS) {
      throw new Error("forecast_columns_invalid");
    }
    const item: ForecastDraftRow = {};
    columns.forEach((column, index) => {
      item[column] = normalizeCell(row[index]);
    });
    return item;
  });

  const dataJson = JSON.stringify(records);
  if (new TextEncoder().encode(dataJson).length > MAX_FORECAST_INLINE_DATA_BYTES) {
    throw new Error("forecast_data_too_large");
  }
  return {
    sourceName,
    columns,
    rowCount: records.length || preview.total_rows,
    dataJson,
    rows: records,
  };
}

function normalizeCell(value: unknown): unknown {
  if (typeof value !== "string") return value;
  const trimmed = value.trim();
  if (trimmed.length > MAX_FORECAST_CELL_CHARS) throw new Error("forecast_cell_too_long");
  if (trimmed === "") return "";
  const numeric = parsePlainNumber(trimmed);
  if (numeric !== null) return numeric;
  return trimmed;
}

function parsePlainNumber(value: string): number | null {
  let commas = 0;
  let dots = 0;
  for (const character of value) {
    if (character === ",") commas += 1;
    else if (character === ".") dots += 1;
    else if (!"0123456789+-eE".includes(character)) return null;
  }
  if (commas > 1 || (commas === 1 && dots > 0)) return null;
  const normalized = commas === 1 ? value.replace(",", ".") : value;
  const numeric = Number(normalized);
  return Number.isFinite(numeric) ? numeric : null;
}

function mapHeaders(headers: string[]): string[] {
  if (headers.length === 0 || headers.length > MAX_FORECAST_INPUT_COLUMNS) {
    throw new Error("forecast_headers_missing");
  }
  const seen = new Set<string>();
  return headers.map((header, index) => {
    if (typeof header !== "string") throw new Error("forecast_headers_invalid");
    const name = header.trim() || `column_${index + 1}`;
    if (name.length > MAX_FORECAST_COLUMN_CHARS) throw new Error("forecast_headers_invalid");
    if (seen.has(name)) throw new Error("forecast_headers_duplicate");
    seen.add(name);
    return name;
  });
}

function basename(path: string): string {
  return path.split(/[\\/]/).pop() || "data";
}
