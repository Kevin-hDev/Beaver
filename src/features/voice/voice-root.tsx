import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { IS_LINUX } from "@/lib/platform";
import { pinComposerDraft, unpinComposerDraft } from "@/hooks/composer-draft-store";
import type { VoiceSnapshot } from "@/types/voice.generated";
import { dispatchVoiceAction, getVoiceSnapshot, VOICE_CHANGED_EVENT } from "./voice-client";
import { deliverVoiceResult } from "./voice-delivery";
import { acceptVoiceSnapshot, readVoiceSnapshot } from "./voice-store";

export function VoiceRoot() {
  useEffect(() => {
    if (IS_LINUX) return;
    let disposed = false;
    let running = false;
    let reread = false;
    let pinned: { key: string; operationId: string } | null = null;

    const reconcile = async () => {
      if (running) { reread = true; return; }
      running = true;
      try {
        do {
          reread = false;
          const snapshot = await getVoiceSnapshot();
          if (disposed) return;
          const current = readVoiceSnapshot();
          if (current && snapshot.revision < current.revision) continue;
          acceptVoiceSnapshot(snapshot);
          const nextPin = reconcilePin(snapshot, pinned);
          pinned = nextPin.pinned;
          if (nextPin.refusedOperationId) {
            const cancelled = await dispatchVoiceAction({
              action: "cancel-insertion",
              operation_id: nextPin.refusedOperationId,
            });
            acceptVoiceSnapshot(cancelled);
            reread = true;
          }
          if (snapshot.delivery) {
            const acknowledged = await deliverVoiceResult(snapshot.delivery);
            if (!disposed) {
              acceptVoiceSnapshot(acknowledged);
              reread = true;
            }
          }
        } while (reread && !disposed);
      } finally {
        running = false;
        if (reread && !disposed) queueMicrotask(() => { void reconcile().catch(() => {}); });
      }
    };

    const unlisten = listen<VoiceSnapshot>(VOICE_CHANGED_EVENT, () => {
      void reconcile().catch(() => {});
    });
    void unlisten.then(() => reconcile()).catch(() => {});
    return () => {
      disposed = true;
      cleanupTauriListener(unlisten);
    };
  }, []);
  return null;
}

function reconcilePin(
  snapshot: VoiceSnapshot,
  previous: { key: string; operationId: string } | null,
): {
  pinned: { key: string; operationId: string } | null;
  refusedOperationId: string | null;
} {
  const destination = snapshot.operation?.destination;
  const next = destination?.kind === "draft"
    ? { key: destination.draft_key, operationId: snapshot.operation!.id }
    : null;
  if (previous && (!next || previous.operationId !== next.operationId)) {
    unpinComposerDraft(previous.key, previous.operationId);
  }
  if (next && (!previous || previous.operationId !== next.operationId)) {
    return pinComposerDraft(next.key, next.operationId)
      ? { pinned: next, refusedOperationId: null }
      : { pinned: null, refusedOperationId: next.operationId };
  }
  return { pinned: next, refusedOperationId: null };
}
