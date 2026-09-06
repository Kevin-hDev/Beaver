const BYTE_BASE = 1024;
const BYTE_UNITS = ["B", "KiB", "MiB", "GiB", "TiB"] as const;
const FRENCH_BYTE_UNITS = ["o", "Kio", "Mio", "Gio", "Tio"] as const;

export function formatByteSize(bytes: number | bigint, locale = "en"): string {
  const numeric = Number(bytes);
  let value = Number.isFinite(numeric) ? Math.max(0, numeric) : 0;
  let unit = 0;
  while (value >= BYTE_BASE && unit < BYTE_UNITS.length - 1) {
    value /= BYTE_BASE;
    unit += 1;
  }
  const units = locale.startsWith("fr") ? FRENCH_BYTE_UNITS : BYTE_UNITS;
  return `${new Intl.NumberFormat(locale, { maximumFractionDigits: unit === 0 ? 0 : 1 }).format(value)} ${units[unit]}`;
}
