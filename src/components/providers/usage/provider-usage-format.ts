export function formatCount(value: number, locale: string): string {
  return new Intl.NumberFormat(locale, { maximumFractionDigits: 2 }).format(value);
}

export function formatUsdMicros(value: number, locale: string): string {
  return new Intl.NumberFormat(locale, {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: value > 0 && value < 10_000 ? 4 : 2,
    maximumFractionDigits: 6,
  }).format(value / 1_000_000);
}

export function formatBalance(amount: string, currency: string, locale: string): string {
  const value = Number(amount);
  if (!Number.isFinite(value)) return `${amount} ${currency}`;
  if (currency === "CREDITS") return `${formatCount(value, locale)} ${currency}`;
  try {
    return new Intl.NumberFormat(locale, {
      style: "currency",
      currency,
      maximumFractionDigits: 6,
    }).format(value);
  } catch {
    return `${amount} ${currency}`;
  }
}

export function formatDate(timestamp: number, locale: string): string {
  const maximum = Math.floor(Date.now() / 1000) + 366 * 24 * 60 * 60;
  if (!Number.isSafeInteger(timestamp) || timestamp < 946_684_800 || timestamp > maximum) {
    return "—";
  }
  try {
    return new Intl.DateTimeFormat(locale, {
      dateStyle: "short",
      timeStyle: "short",
    }).format(new Date(timestamp * 1000));
  } catch {
    return "—";
  }
}
