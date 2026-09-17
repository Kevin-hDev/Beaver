import { useSyncExternalStore } from "react";
import type { VoiceSnapshot } from "@/types/voice.generated";
import { addBoundedSubscriber } from "@/lib/bounded-subscriber";

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

export function subscribeVoiceSnapshots(listener: () => void): () => void {
  const id = nextListenerId++;
  return addBoundedSubscriber(listeners, id, listener, MAX_LISTENERS, "voice-snapshots");
}

export function useVoiceSnapshot(): VoiceSnapshot | null {
  return useSyncExternalStore(subscribeVoiceSnapshots, readVoiceSnapshot, readVoiceSnapshot);
}

export function resetVoiceStoreForTests() { snapshot = null; listeners.clear(); }
