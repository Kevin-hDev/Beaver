import type { ManagedStreamState } from "./agent-chat-stream-callbacks";
import { addBoundedSubscriber } from "@/lib/bounded-subscriber";

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
  const id = nextSubscriberId++;
  return addBoundedSubscriber(subscribers, id, subscriber, MAX_ACTIVITY_SUBSCRIBERS);
}
