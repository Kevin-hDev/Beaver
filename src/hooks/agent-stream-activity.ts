import type { ManagedStreamState } from "./agent-chat-stream-callbacks";

const MAX_ACTIVITY_SUBSCRIBERS = 16;

export interface StreamActivity {
  sessionId: string;
  isStreaming: boolean;
  completed: boolean;
  updatedAt: number;
}

type ActivitySubscriber = (activity: StreamActivity) => void;

const subscribers = new Map<number, ActivitySubscriber>();
let nextSubscriberId = 1;

export function toStreamActivity(sessionId: string, state: ManagedStreamState): StreamActivity {
  return {
    sessionId,
    isStreaming: state.isStreaming,
    completed: state.completed,
    updatedAt: state.updatedAt,
  };
}

export function emitStreamActivity(sessionId: string, state: ManagedStreamState) {
  const activity = toStreamActivity(sessionId, state);
  for (const subscriber of subscribers.values()) subscriber(activity);
}

export function subscribeStreamActivity(subscriber: ActivitySubscriber): () => void {
  while (subscribers.size >= MAX_ACTIVITY_SUBSCRIBERS) {
    const first = subscribers.keys().next().value;
    if (first === undefined) break;
    subscribers.delete(first);
  }
  const id = nextSubscriberId++;
  subscribers.set(id, subscriber);
  return () => {
    subscribers.delete(id);
  };
}
