import { useSyncExternalStore } from "react";
import type { VoiceSnapshot } from "@/types/voice.generated";

const MAX_LISTENERS = 64;
let snapshot: VoiceSnapshot | null = null;
let nextListenerId = 1;
const listeners = new Map<number, () => void>();

export function acceptVoiceSnapshot(next: VoiceSnapshot): boolean {
  if (snapshot && next.revision <= snapshot.revision) return false;
  snapshot = next;
  for (const listener of listeners.values()) listener();
  return true;
}

export function readVoiceSnapshot(): VoiceSnapshot | null { return snapshot; }

function subscribe(listener: () => void): () => void {
  while (listeners.size >= MAX_LISTENERS) {
    const oldest = listeners.keys().next().value;
    if (oldest === undefined) break;
    listeners.delete(oldest);
  }
  const id = nextListenerId++;
  listeners.set(id, listener);
  return () => { listeners.delete(id); };
}

export function useVoiceSnapshot(): VoiceSnapshot | null {
  return useSyncExternalStore(subscribe, readVoiceSnapshot, readVoiceSnapshot);
}

export function resetVoiceStoreForTests() { snapshot = null; }
