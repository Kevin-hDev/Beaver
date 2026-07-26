const YEAR_OPTIONS: Intl.DateTimeFormatOptions = { year: "numeric" };
const MONTH_OPTIONS: Intl.DateTimeFormatOptions = { month: "short", year: "2-digit" };
const DAY_OPTIONS: Intl.DateTimeFormatOptions = { month: "2-digit", day: "2-digit" };

/**
 * Granularity-aware x-axis label: ECharts time ticks land on clean
 * boundaries, so the date itself tells which label fits — yearly ticks
 * fall on January 1st, monthly ticks on the 1st, finer ticks on days.
 */
export function formatAxisDate(value: number, locale: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return "";
  if (parsed.getMonth() === 0 && parsed.getDate() === 1) {
    return new Intl.DateTimeFormat(locale, YEAR_OPTIONS).format(parsed);
  }
  if (parsed.getDate() === 1) {
    return new Intl.DateTimeFormat(locale, MONTH_OPTIONS).format(parsed);
  }
  return new Intl.DateTimeFormat(locale, DAY_OPTIONS).format(parsed);
}
