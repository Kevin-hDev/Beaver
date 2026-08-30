export function formatTokenCount(tokens: number): string {
  if (tokens >= 1_000_000) return compact(tokens, 1_000_000, "M");
  if (tokens >= 1_000) return compact(tokens, 1_000, "K");
  return String(tokens);
}

function compact(value: number, divisor: number, suffix: string): string {
  const scaled = value / divisor;
  const formatted = Number.isInteger(scaled) ? String(scaled) : scaled.toFixed(1);
  return `${formatted}${suffix}`;
}
