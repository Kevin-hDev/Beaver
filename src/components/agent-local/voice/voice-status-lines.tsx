import { useTranslation } from "react-i18next";
import { showToast } from "@/lib/toast-emitter";
import { voiceNotes } from "@/features/voice/voice-notes";
import { dispatchVoiceAction } from "@/features/voice/voice-client";
import { acceptVoiceSnapshot } from "@/features/voice/voice-store";
import type { VoiceSnapshot } from "@/types/voice.generated";
import { useVoiceSnapshot } from "@/features/voice/voice-store";
import "./voice-controls.css";

export function VoiceStatusLines({ snapshot: provided, draftKey }: { snapshot?: VoiceSnapshot | null; draftKey: string }) {
  const { t } = useTranslation();
  const current = useVoiceSnapshot();
  const snapshot = provided === undefined ? current : provided;
  const notes = voiceNotes(snapshot, draftKey);
  const recovery = snapshot?.recovery;
  const visibleRecovery = recovery && (recovery.draftKey === draftKey || recovery.draftKey === null) ? recovery : null;
  const dispatch = (action: Parameters<typeof dispatchVoiceAction>[0]) => {
    void dispatchVoiceAction(action).then(acceptVoiceSnapshot)
      .catch(() => showToast(t("errors.operationFailed"), "error"));
  };
  if (notes.length === 0) return null;
  return <div className="vc-notes" aria-live="polite">{notes.map((note) => (
    <div key={note.kind} className={`vc-note vc-note-${note.kind}`}>{t(note.key)}</div>
  ))}{visibleRecovery && <div className="vc-recovery-actions">
    {visibleRecovery.status === "ready" && <button type="button" className="btn btn-sm btn-secondary" onClick={() => dispatch({ action: "restore-recovery", recovery_id: visibleRecovery.id, draft_key: draftKey })}>{t("voice.recovery.restore", { duration: t("voice.recovery.duration", durationParts(visibleRecovery.captureMs)) })}</button>}
    <button type="button" className="btn btn-sm btn-destructive" onClick={() => dispatch({ action: "delete-recovery", recovery_id: visibleRecovery.id })}>{t("voice.recovery.delete")}</button>
  </div>}</div>;
}

function durationParts(milliseconds: number) {
  const seconds = Math.floor(milliseconds / 1000);
  return { minutes: Math.floor(seconds / 60), seconds: String(seconds % 60).padStart(2, "0") };
}
