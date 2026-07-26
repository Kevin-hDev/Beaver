export interface ForecastWorkbenchContext {
  session_id: string;
  analysis_id: string | null;
  revision: number;
}

export interface ForecastWorkbenchSnapshot {
  context: ForecastWorkbenchContext;
  draft: ForecastWorkbenchDraft;
  analysis_name: string | null;
}

export interface ForecastWorkbenchDraft {
  section: ForecastWorkbenchSection;
  revision: number;
}

export interface ForecastWorkbenchGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type ForecastWorkbenchSection =
  | "data"
  | "forecast"
  | "evaluation"
  | "comparison"
  | "scenarios"
  | "notes"
  | "report";

export function isForecastWorkbenchSection(value: unknown): value is ForecastWorkbenchSection {
  return ["data", "forecast", "evaluation", "comparison", "scenarios", "notes", "report"]
    .includes(String(value));
}

export function isForecastWorkbenchSnapshot(
  value: unknown,
): value is ForecastWorkbenchSnapshot {
  if (!value || typeof value !== "object") return false;
  const snapshot = value as Partial<ForecastWorkbenchSnapshot>;
  const context = snapshot.context as Partial<ForecastWorkbenchContext> | undefined;
  const validId = (id: unknown) => typeof id === "string" &&
    id.length > 0 && id.length <= 64 && /^[a-f0-9-]+$/.test(id);
  return Boolean(context) &&
    validId(context?.session_id) &&
    (context?.analysis_id === null || validId(context?.analysis_id)) &&
    typeof context?.revision === "number" &&
    Number.isSafeInteger(context.revision) &&
    context.revision > 0 &&
    isForecastWorkbenchSection(snapshot.draft?.section) &&
    Number.isSafeInteger(snapshot.draft?.revision) &&
    Number(snapshot.draft?.revision) > 0 &&
    (snapshot.analysis_name === null || (
      typeof snapshot.analysis_name === "string" && snapshot.analysis_name.length <= 120
    ));
}
