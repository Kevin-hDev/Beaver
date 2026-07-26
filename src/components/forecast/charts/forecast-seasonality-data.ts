export interface SeasonalityPoint {
  date: string;
  value: number;
}

export interface SeasonalityYear {
  year: number;
  values: (number | null)[];
  complete: boolean;
  emphasized: boolean;
}

export interface SeasonalityModel {
  periods: string[];
  years: SeasonalityYear[];
}

export const SEASONALITY_MIN_YEAR_POINTS = 3;
/** Gate for rendering the seasonality card at all (> 24 history points). */
export const SEASONALITY_MIN_HISTORY = 24;
/** Complete years shown by default (plus the current partial year). */
export const SEASONALITY_DEFAULT_VISIBLE_YEARS = 5;
const MONTHS_PER_YEAR = 12;
const QUARTERS_PER_YEAR = 4;

interface SeasonalityCadence {
  periods: string[];
  completePoints: number;
  aggregateAll: boolean;
  bucketForMonth: (month: number) => number;
}

/**
 * Calendar year/month of an observation. Date-only strings ("YYYY-MM-DD")
 * and ISO timestamps are parsed from the string itself: `new Date()` would
 * interpret date-only strings as UTC midnight and shift them into the
 * previous month/year in timezones behind UTC.
 */
function parseYearMonth(date: string): { year: number; month: number } | null {
  const iso = /^(\d{4})-(\d{2})-\d{2}/.exec(date);
  if (iso) {
    const month = Number(iso[2]) - 1;
    return month >= 0 && month <= 11 ? { year: Number(iso[1]), month } : null;
  }
  const parsed = new Date(date);
  if (Number.isNaN(parsed.getTime())) return null;
  return { year: parsed.getFullYear(), month: parsed.getMonth() };
}

/**
 * Default visible subset: the last SEASONALITY_DEFAULT_VISIBLE_YEARS
 * complete years, plus the trailing partial year when there is one.
 */
export function defaultVisibleYears(years: SeasonalityYear[]): number[] {
  const visible = years
    .filter((entry) => entry.complete)
    .slice(-SEASONALITY_DEFAULT_VISIBLE_YEARS)
    .map((entry) => entry.year);
  const last = years[years.length - 1];
  if (last && !last.complete && !visible.includes(last.year)) {
    visible.push(last.year);
  }
  return visible;
}

export function toggleVisibleYear(visible: number[], year: number): number[] {
  if (visible.includes(year)) return visible.filter((entry) => entry !== year);
  return [...visible, year].sort((left, right) => left - right);
}

export function supportsSeasonalityFrequency(frequency?: string): boolean {
  return frequency?.toUpperCase() !== "Y";
}

function resolveCadence(locale: string, frequency?: string): SeasonalityCadence | null {
  const normalized = frequency?.toUpperCase();
  if (normalized === "Y") return null;
  if (normalized === "Q") {
    return {
      periods: ["Q1", "Q2", "Q3", "Q4"],
      completePoints: QUARTERS_PER_YEAR,
      aggregateAll: false,
      bucketForMonth: (month) => Math.floor(month / 3),
    };
  }
  return {
    periods: monthNames(locale),
    completePoints: MONTHS_PER_YEAR,
    aggregateAll: frequency != null && normalized !== "M",
    bucketForMonth: (month) => month,
  };
}

/**
 * Groups history by calendar year and normalizes each year to its first
 * available period (= 100), so recurring patterns line up on one axis.
 * Years with fewer than SEASONALITY_MIN_YEAR_POINTS points are skipped.
 * Sub-monthly frequencies aggregate observations of a month by mean.
 * Quarterly inputs use four quarter buckets. Annual inputs are unsupported:
 * one observation per year cannot reveal within-year seasonality.
 */
export function buildSeasonalityModel(
  points: SeasonalityPoint[],
  locale: string,
  frequency?: string,
): SeasonalityModel | null {
  const cadence = resolveCadence(locale, frequency);
  if (!cadence) return null;
  const byYear = new Map<number, Map<number, number[]>>();
  for (const point of points) {
    if (!Number.isFinite(point.value)) continue;
    const yearMonth = parseYearMonth(point.date);
    if (!yearMonth) continue;
    const entry = byYear.get(yearMonth.year) ?? new Map<number, number[]>();
    const period = cadence.bucketForMonth(yearMonth.month);
    const bucket = entry.get(period) ?? [];
    bucket.push(point.value);
    entry.set(period, bucket);
    byYear.set(yearMonth.year, entry);
  }

  const years: SeasonalityYear[] = [];
  for (const [year, periods] of [...byYear.entries()].sort((a, b) => a[0] - b[0])) {
    if (periods.size < SEASONALITY_MIN_YEAR_POINTS) continue;
    const resolve = (bucket: number[]) =>
      cadence.aggregateAll
        ? bucket.reduce((sum, value) => sum + value, 0) / bucket.length
        : bucket[0];
    const ordered = [...periods.entries()].sort((a, b) => a[0] - b[0]);
    const base = resolve(ordered[0][1]);
    if (!Number.isFinite(base) || base === 0) continue;
    const values = Array.from({ length: cadence.periods.length }, (_, period) => {
      const bucket = periods.get(period);
      return bucket == null ? null : (resolve(bucket) / base) * 100;
    });
    years.push({
      year,
      values,
      complete: periods.size >= cadence.completePoints,
      emphasized: false,
    });
  }
  if (!years.length) return null;

  const emphasized =
    [...years].reverse().find((entry) => entry.complete) ?? years[years.length - 1];
  emphasized.emphasized = true;

  return { periods: cadence.periods, years };
}

export function monthNames(locale: string): string[] {
  const formatter = new Intl.DateTimeFormat(locale, { month: "short" });
  return Array.from({ length: 12 }, (_, month) =>
    formatter.format(new Date(2024, month, 1)),
  );
}
