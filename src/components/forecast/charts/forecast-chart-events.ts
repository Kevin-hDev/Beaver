export function readDataIndex(event: unknown): number | null {
  return typeof event === "object" &&
    event !== null &&
    "dataIndex" in event &&
    typeof (event as { dataIndex?: unknown }).dataIndex === "number"
    ? (event as { dataIndex: number }).dataIndex
    : null;
}

export function readSeriesId(event: unknown): string | null {
  return typeof event === "object" &&
    event !== null &&
    "seriesId" in event &&
    typeof (event as { seriesId?: unknown }).seriesId === "string"
    ? (event as { seriesId: string }).seriesId
    : null;
}

export function readFirstAnnotationId(event: unknown): string | null {
  if (typeof event !== "object" || event === null || !("data" in event)) return null;
  const data = (event as { data?: unknown }).data;
  if (typeof data !== "object" || data === null || !("annotationIds" in data)) return null;
  const ids = (data as { annotationIds?: unknown }).annotationIds;
  return Array.isArray(ids) && typeof ids[0] === "string" ? ids[0] : null;
}

export function dateKey(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value.slice(0, 10);
  return parsed.toISOString().slice(0, 10);
}
