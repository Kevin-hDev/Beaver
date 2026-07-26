const MINUTE_MS = 60_000;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;
const MONTH_MS = 30 * DAY_MS;
const YEAR_MS = 365 * DAY_MS;

type SessionAgeUnit = "minute" | "hour" | "day" | "month" | "year";

export interface SessionAge {
  count: number;
  unit: SessionAgeUnit;
}

export function getSessionAge(createdAt: string, nowMs = Date.now()): SessionAge | null {
  const createdMs = new Date(createdAt).getTime();
  if (!Number.isFinite(createdMs)) return null;

  const elapsedMs = Math.max(0, nowMs - createdMs);
  if (elapsedMs < HOUR_MS) {
    return { count: Math.max(1, Math.floor(elapsedMs / MINUTE_MS)), unit: "minute" };
  }
  if (elapsedMs < DAY_MS) {
    return { count: Math.floor(elapsedMs / HOUR_MS), unit: "hour" };
  }
  if (elapsedMs < MONTH_MS) {
    return { count: Math.floor(elapsedMs / DAY_MS), unit: "day" };
  }
  if (elapsedMs < YEAR_MS) {
    return { count: Math.floor(elapsedMs / MONTH_MS), unit: "month" };
  }
  return { count: Math.floor(elapsedMs / YEAR_MS), unit: "year" };
}
