import type { VoiceSnapshot } from "@/types/voice.generated";

export type VoiceNoteKind = "activity" | "recovery" | "error";
export interface VoiceNote { kind: VoiceNoteKind; key: string }

// Les statuts d'activité (préparation, transcription…) ne sont pas des notes :
// ils s'affichent dans le placeholder ou la rangée de contrôles, sans ajouter
// de ligne — le champ de saisie ne doit jamais changer de hauteur en dictée.
export function voiceNotes(snapshot: VoiceSnapshot | null, draftKey: string): VoiceNote[] {
  if (!snapshot) return [];
  const notes: VoiceNote[] = [];
  if (snapshot.recovery && (snapshot.recovery.draftKey === draftKey || snapshot.recovery.draftKey === null)) {
    notes.push({ kind: "recovery", key: `voice.recovery.${snapshot.recovery.status}` });
  }
  if (snapshot.error) notes.push({ kind: "error", key: `voice.error.${snapshot.error.code}` });
  return notes;
}
