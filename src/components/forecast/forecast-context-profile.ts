import type { ForecastDraftRow } from "./forecast-data";

export interface ForecastContextProfile {
  historyRows: number;
  futureRows: number;
  seriesCount: number;
  futureRowsPerSeries: number | null;
  selectedCovariates: number;
  futureContextColumns: string[];
}

export function buildForecastContextProfile(
  rows: ForecastDraftRow[],
  targetColumn: string,
  covariates: string[],
  seriesColumn: string | null,
): ForecastContextProfile {
  const futureContextColumns = new Set<string>();
  const seriesIds = new Set<string>();
  const futureCounts = new Map<string, number>();
  let historyRows = 0;
  let futureRows = 0;

  rows.forEach((row) => {
    const seriesId = readSeriesId(row, seriesColumn);
    if (seriesId) {
      seriesIds.add(seriesId);
    }
    if (hasValue(row[targetColumn])) {
      historyRows += 1;
      return;
    }

    futureRows += 1;
    if (seriesId) {
      futureCounts.set(seriesId, (futureCounts.get(seriesId) ?? 0) + 1);
    }
    covariates.forEach((column) => {
      if (hasValue(row[column])) {
        futureContextColumns.add(column);
      }
    });
  });

  return {
    historyRows,
    futureRows,
    seriesCount: seriesIds.size || 1,
    futureRowsPerSeries: inferFutureRowsPerSeries(futureCounts),
    selectedCovariates: covariates.length,
    futureContextColumns: [...futureContextColumns],
  };
}

function hasValue(value: unknown): boolean {
  if (value === null || value === undefined) return false;
  if (typeof value === "number") return Number.isFinite(value);
  if (typeof value === "string") return value.trim().length > 0;
  if (typeof value === "boolean") return true;
  return true;
}

function inferFutureRowsPerSeries(futureCounts: Map<string, number>): number | null {
  if (futureCounts.size === 0) return null;
  const values = [...futureCounts.values()];
  const first = values[0];
  return values.every((value) => value === first) ? first : null;
}

function readSeriesId(row: ForecastDraftRow, seriesColumn: string | null): string | null {
  if (!seriesColumn) return null;
  const value = row[seriesColumn];
  if (typeof value === "string") {
    const trimmed = value.trim();
    return trimmed || null;
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  return null;
}
