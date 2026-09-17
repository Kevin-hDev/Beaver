const LIMIT_ERROR = "Active view subscription limit reached";

export function addBoundedSubscriber<T>(
  subscribers: Map<number, T>,
  id: number,
  subscriber: T,
  limit: number,
): () => void {
  if (subscribers.size >= limit) throw new RangeError(LIMIT_ERROR);
  subscribers.set(id, subscriber);
  return () => {
    subscribers.delete(id);
  };
}
