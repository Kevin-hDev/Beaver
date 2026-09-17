import { error as logError } from "@tauri-apps/plugin-log";

const LIMIT_ERROR = "active_view_subscription_limit_reached";

export function addBoundedSubscriber<T>(
  subscribers: Map<number, T>,
  id: number,
  subscriber: T,
  limit: number,
  registry: string,
): () => void {
  if (subscribers.size >= limit) {
    void logError(LIMIT_ERROR, {
      keyValues: { registry, size: String(subscribers.size), limit: String(limit) },
    }).catch(() => console.error(LIMIT_ERROR, { registry, size: subscribers.size, limit }));
    throw new RangeError(LIMIT_ERROR);
  }
  subscribers.set(id, subscriber);
  return () => {
    subscribers.delete(id);
  };
}
