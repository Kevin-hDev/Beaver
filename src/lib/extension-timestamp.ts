export function isExtensionTimestamp(value: unknown): value is string {
  if (typeof value !== "string" || value.length < 20 || value.length > 64) return false;
  if (value[4] !== "-" || value[7] !== "-" || value[10] !== "T"
    || value[13] !== ":" || value[16] !== ":") return false;
  if (!digits(value, 0, 4) || !digits(value, 5, 7) || !digits(value, 8, 10)
    || !digits(value, 11, 13) || !digits(value, 14, 16) || !digits(value, 17, 19)) return false;
  const zoneStart = value.endsWith("Z") ? value.length - 1 : value.length - 6;
  if (zoneStart < 19 || !validFraction(value, zoneStart) || !validZone(value, zoneStart)) {
    return false;
  }
  return Number.isFinite(Date.parse(value));
}

function validFraction(value: string, zoneStart: number): boolean {
  if (zoneStart === 19) return true;
  return value[19] === "." && zoneStart > 20 && zoneStart <= 29
    && digits(value, 20, zoneStart);
}

function validZone(value: string, start: number): boolean {
  if (value[start] === "Z") return start === value.length - 1;
  return (value[start] === "+" || value[start] === "-")
    && value[start + 3] === ":"
    && digits(value, start + 1, start + 3)
    && digits(value, start + 4, start + 6);
}

function digits(value: string, start: number, end: number): boolean {
  for (let index = start; index < end; index += 1) {
    const code = value.charCodeAt(index);
    if (code < 48 || code > 57) return false;
  }
  return true;
}
