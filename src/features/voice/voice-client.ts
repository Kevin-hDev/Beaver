import { invoke } from "@tauri-apps/api/core";
import type { VoiceAction, VoiceSnapshot } from "@/types/voice.generated";

export const VOICE_CHANGED_EVENT = "voice-state-changed";

export function getVoiceSnapshot(): Promise<VoiceSnapshot> {
  return invoke<VoiceSnapshot>("voice_get_snapshot");
}

export function dispatchVoiceAction(action: VoiceAction): Promise<VoiceSnapshot> {
  return invoke<VoiceSnapshot>("voice_dispatch", { action });
}
