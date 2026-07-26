import { invoke } from "@tauri-apps/api/core";
import type { PersonalityFile } from "@/types/personality";

export async function listFiles(): Promise<PersonalityFile[]> {
  return invoke<PersonalityFile[]>("list_personality_files");
}

export async function readFile(path: string): Promise<string> {
  return invoke<string>("read_personality_file", { path });
}

export async function openInEditor(path: string): Promise<void> {
  return invoke("open_in_editor", { path });
}

export async function getInjectionState(): Promise<Record<string, boolean>> {
  return invoke<Record<string, boolean>>("get_injection_state");
}

export async function setInjectionState(name: string, enabled: boolean): Promise<void> {
  return invoke("set_injection_state", { name, enabled });
}
