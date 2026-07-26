interface RevisionedForecast {
  id?: string;
  revision?: number;
}

export function preferNewestForecast<T extends RevisionedForecast>(
  current: T | null,
  next: T,
): T {
  if (
    current?.id &&
    next.id &&
    current.id !== next.id
  ) {
    return next;
  }
  if (
    current &&
    Number.isSafeInteger(current.revision) &&
    Number.isSafeInteger(next.revision) &&
    Number(next.revision) < Number(current.revision)
  ) {
    return current;
  }
  return next;
}
