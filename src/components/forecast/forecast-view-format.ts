export interface ForecastMetricMeta {
  columnTitle: string;
  unitLabel: string | null;
  unitKind: "currency-eur" | "count" | "generic";
}

export interface ForecastPeriodMeta {
  stepLabel: string;
  secondaryLabel: string;
}

const DATE_FREQUENCIES = new Set(["S", "T", "MIN", "H", "D", "B", "W", "M", "Q", "Y"]);
/** Monthly or coarser: any time component is a timezone artifact, not data. */
const TIMELESS_FREQUENCIES = new Set(["M", "Q", "Y"]);

export function inferMetricMeta(
  locale: string,
  targetColumn?: string,
  fallbackName?: string
): ForecastMetricMeta {
  const raw = (targetColumn || stripForecastPrefix(fallbackName || "")).trim();
  const normalized = raw.toLowerCase();
  if (looksLikeCurrency(normalized)) {
    return {
      columnTitle: prettifyMetricName(raw || "forecast"),
      unitLabel: locale.startsWith("fr") ? "€" : "EUR",
      unitKind: "currency-eur",
    };
  }
  if (looksLikeCount(normalized)) {
    return {
      columnTitle: prettifyMetricName(raw || "forecast"),
      unitLabel: locale.startsWith("fr") ? "unités" : "units",
      unitKind: "count",
    };
  }
  return {
    columnTitle: prettifyMetricName(raw || "forecast"),
    unitLabel: null,
    unitKind: "generic",
  };
}

export function formatForecastValue(
  value: number,
  locale: string,
  metric: ForecastMetricMeta
): string {
  if (metric.unitKind === "currency-eur") {
    return new Intl.NumberFormat(locale, {
      style: "currency",
      currency: "EUR",
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);
  }
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(value);
}

export function buildPeriodMeta(
  index: number,
  rawDate: string,
  endDate: string,
  frequency: string,
  locale: string
): ForecastPeriodMeta {
  const stepLabel = `T+${index + 1}`;
  const resolvedDate = resolvePredictionDate(rawDate, endDate, frequency, index);
  if (!resolvedDate) {
    return { stepLabel, secondaryLabel: rawDate };
  }
  return {
    stepLabel,
    secondaryLabel: formatResolvedDate(resolvedDate, frequency, locale),
  };
}

export function resolvePredictionDate(
  rawDate: string,
  endDate: string,
  frequency: string,
  index: number
): Date | null {
  const explicit = parseDate(rawDate);
  if (explicit) return explicit;
  if (!/^T\+\d+$/i.test(rawDate.trim())) return null;
  if (!DATE_FREQUENCIES.has(frequency.trim().toUpperCase())) return null;
  const base = parseDate(endDate);
  if (!base) return null;
  const step = index + 1;
  return shiftDate(base, frequency.trim().toUpperCase(), step);
}

function shiftDate(base: Date, frequency: string, step: number): Date | null {
  const next = new Date(base.getTime());
  switch (frequency) {
    case "S":
      next.setSeconds(next.getSeconds() + step);
      return next;
    case "T":
    case "MIN":
      next.setMinutes(next.getMinutes() + step);
      return next;
    case "H":
      next.setHours(next.getHours() + step);
      return next;
    case "D":
    case "B":
      next.setDate(next.getDate() + step);
      return next;
    case "W":
      next.setDate(next.getDate() + step * 7);
      return next;
    case "M":
      next.setMonth(next.getMonth() + step);
      return next;
    case "Q":
      next.setMonth(next.getMonth() + step * 3);
      return next;
    case "Y":
      next.setFullYear(next.getFullYear() + step);
      return next;
    default:
      return null;
  }
}

function parseDate(value: string): Date | null {
  if (!value.trim()) return null;
  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime()) ? null : parsed;
}

function formatResolvedDate(date: Date, frequency: string, locale: string): string {
  const hasTime = !TIMELESS_FREQUENCIES.has(frequency.trim().toUpperCase())
    && (date.getHours() !== 0 || date.getMinutes() !== 0 || date.getSeconds() !== 0);
  const weekday = new Intl.DateTimeFormat(locale, { weekday: "long" }).format(date);
  const day = new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(date);
  if (!hasTime) return `${capitalize(weekday)} - ${day}`;
  const time = new Intl.DateTimeFormat(locale, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
  return `${capitalize(weekday)} - ${day} - ${time}`;
}

function looksLikeCurrency(value: string): boolean {
  return ["eur", "euro", "ca", "revenue", "turnover", "sales_eur"].some((token) => value.includes(token));
}

function looksLikeCount(value: string): boolean {
  return ["pizza", "pizzas", "qty", "quantity", "items", "units", "orders", "commandes", "nb_"].some((token) => value.includes(token));
}

function prettifyMetricName(value: string): string {
  return value
    .replace(/^forecast\s+/i, "")
    .replace(/_/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

function stripForecastPrefix(value: string): string {
  return value.replace(/^forecast\s+/i, "");
}

function capitalize(value: string): string {
  return value ? value[0].toUpperCase() + value.slice(1) : value;
}
