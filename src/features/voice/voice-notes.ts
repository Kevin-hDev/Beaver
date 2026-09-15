import type { VoiceSnapshot } from "@/types/voice.generated";

export type VoiceNoteKind = "activity" | "recovery" | "error";
export interface VoiceNote { kind: VoiceNoteKind; key: string }

export function voiceNotes(snapshot: VoiceSnapshot | null, draftKey: string): VoiceNote[] {
  if (!snapshot) return [];
  const notes: VoiceNote[] = [];
  const destination = snapshot.operation?.destination;
  if (destination?.kind === "draft" && destination.draft_key === draftKey) {
    notes.push({ kind: "activity", key: `voice.status.${snapshot.phase}` });
  }
  if (snapshot.recovery?.draftKey === draftKey) {
    notes.push({ kind: "recovery", key: `voice.recovery.${snapshot.recovery.status}` });
  }
  if (snapshot.error) notes.push({ kind: "error", key: `voice.error.${snapshot.error.code}` });
  return notes;
}
