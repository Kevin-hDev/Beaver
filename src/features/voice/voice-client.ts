import { invoke } from "@tauri-apps/api/core";
import type { VoiceAction, VoiceDevice, VoiceSettings, VoiceSettingsPatch, VoiceSnapshot } from "@/types/voice.generated";

export const VOICE_CHANGED_EVENT = "voice-state-changed";

export function getVoiceSnapshot(): Promise<VoiceSnapshot> {
  return invoke<VoiceSnapshot>("voice_get_snapshot");
}

export function dispatchVoiceAction(action: VoiceAction): Promise<VoiceSnapshot> {
  return invoke<VoiceSnapshot>("voice_dispatch", { action });
}

export function getVoiceSettings(): Promise<VoiceSettings> {
  return invoke<VoiceSettings>("voice_get_settings");
}

export function updateVoiceSettings(patch: VoiceSettingsPatch): Promise<VoiceSettings> {
  return invoke<VoiceSettings>("voice_update_settings", { patch });
}

export function listVoiceDevices(): Promise<VoiceDevice[]> {
  return invoke<VoiceDevice[]>("voice_list_devices");
}
