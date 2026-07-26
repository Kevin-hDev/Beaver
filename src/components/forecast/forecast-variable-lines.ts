import type { ForecastDraftRow } from "./forecast-data";

export interface ForecastVariableLine {
  id: string;
  name: string;
  values: Array<number | null>;
  rawValues: Array<number | null>;
}

interface BuildVariableLinesArgs {
  rows: ForecastDraftRow[];
  covariates: string[];
  targetColumn: string;
  seriesColumn?: string | null;
  selectedSeries?: string;
  historyValues: number[];
  forecastValues: number[];
}

export function buildForecastVariableLines(
  args: BuildVariableLinesArgs,
): ForecastVariableLine[] {
  if (!args.covariates.length || !args.rows.length) return [];

  const seriesRows = filterRowsBySeries(args.rows, args.seriesColumn ?? null, args.selectedSeries);
  const historyRows = seriesRows.filter((row) => hasValue(row[args.targetColumn]));
  const futureRows = seriesRows.filter((row) => !hasValue(row[args.targetColumn]));
  const targetValues = [...args.historyValues, ...args.forecastValues].filter(Number.isFinite);
  const targetRange = range(targetValues);

  return args.covariates
    .map((name) => {
      const rawValues = [
        ...historyRows.map((row) => readNumeric(row[name])),
        ...futureRows.map((row) => readNumeric(row[name])),
      ];
      if (!rawValues.some((value) => value != null)) return null;
      return {
        id: `variable-${name}`,
        name,
        rawValues,
        values: normalizeValues(rawValues, targetRange),
      } satisfies ForecastVariableLine;
    })
    .filter((line): line is ForecastVariableLine => Boolean(line));
}

function filterRowsBySeries(
  rows: ForecastDraftRow[],
  seriesColumn: string | null,
  selectedSeries?: string,
) {
  if (!seriesColumn || !selectedSeries) return rows;
  return rows.filter((row) => stringifyValue(row[seriesColumn]) === selectedSeries);
}

function normalizeValues(
  values: Array<number | null>,
  targetRange: [number, number],
): Array<number | null> {
  const variableRange = range(values.filter((value): value is number => value != null));
  const [targetMin, targetMax] = targetRange;
  const [variableMin, variableMax] = variableRange;
  const targetSpan = targetMax - targetMin;
  const variableSpan = variableMax - variableMin;
  const mid = targetMin + targetSpan / 2;

  return values.map((value) => {
    if (value == null) return null;
    if (!Number.isFinite(value)) return null;
    if (targetSpan <= 0 || variableSpan <= 0) return mid;
    return targetMin + ((value - variableMin) / variableSpan) * targetSpan;
  });
}

function range(values: number[]): [number, number] {
  if (!values.length) return [0, 1];
  let min = values[0];
  let max = values[0];
  for (const value of values) {
    if (value < min) min = value;
    if (value > max) max = value;
  }
  if (min === max) return [min - 1, max + 1];
  return [min, max];
}

function readNumeric(value: unknown): number | null {
  if (typeof value === "number") return Number.isFinite(value) ? value : null;
  if (typeof value === "string") {
    const parsed = Number(value.trim().replace(",", "."));
    return Number.isFinite(parsed) ? parsed : null;
  }
  if (typeof value === "boolean") return value ? 1 : 0;
  return null;
}

function hasValue(value: unknown): boolean {
  if (value == null) return false;
  if (typeof value === "string") return value.trim().length > 0;
  if (typeof value === "number") return Number.isFinite(value);
  if (typeof value === "boolean") return true;
  return true;
}

function stringifyValue(value: unknown): string | null {
  if (typeof value === "string") return value.trim() || null;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return null;
}
