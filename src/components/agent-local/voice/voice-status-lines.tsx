import { useTranslation } from "react-i18next";
import { voiceNotes } from "@/features/voice/voice-notes";
import type { VoiceSnapshot } from "@/types/voice.generated";
import { useVoiceSnapshot } from "@/features/voice/voice-store";
import "./voice-controls.css";

export function VoiceStatusLines({ snapshot: provided, draftKey }: { snapshot?: VoiceSnapshot | null; draftKey: string }) {
  const { t } = useTranslation();
  const current = useVoiceSnapshot();
  const snapshot = provided === undefined ? current : provided;
  const notes = voiceNotes(snapshot, draftKey);
  if (notes.length === 0) return null;
  return <div className="vc-notes" aria-live="polite">{notes.map((note) => (
    <div key={note.kind} className={`vc-note vc-note-${note.kind}`}>{t(note.key)}</div>
  ))}</div>;
}
