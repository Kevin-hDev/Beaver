import {
  closeComposerDraft,
  openComposerDraft,
  WELCOME_COMPOSER_DRAFT_KEY,
} from "@/hooks/composer-draft-store";
import { dispatchVoiceAction } from "./voice-client";
import { acceptVoiceSnapshot } from "./voice-store";
import { IS_LINUX } from "@/lib/platform";

async function closeVoiceDraft(draftKey: string): Promise<void> {
  closeComposerDraft(draftKey);
  if (IS_LINUX) return;
  const next = await dispatchVoiceAction({
    action: "destination-closed",
    destination: { kind: "draft", draft_key: draftKey },
  });
  acceptVoiceSnapshot(next);
}

export async function notifyVoiceMessageAccepted(draftKey: string, sendId: string): Promise<void> {
  const closesWelcome = draftKey === WELCOME_COMPOSER_DRAFT_KEY;
  if (closesWelcome) closeComposerDraft(draftKey);
  if (IS_LINUX) {
    return;
  }
  let failure: unknown;
  try {
    const accepted = await dispatchVoiceAction({
      action: "message-accepted",
      draft_key: draftKey,
      send_id: sendId,
    });
    acceptVoiceSnapshot(accepted);
  } catch (error) {
    failure = error;
  }
  if (closesWelcome) {
    try { await closeVoiceDraft(draftKey); } catch (error) { failure ??= error; }
  }
  if (failure) throw failure instanceof Error
    ? failure
    : new Error("voice_message_notification_failed");
}

export async function closeVoiceDraftWhile<T>(draftKey: string, close: () => Promise<T>): Promise<T> {
  await closeVoiceDraft(draftKey).catch(() => {});
  try {
    return await close();
  } catch (error) {
    openComposerDraft(draftKey);
    throw error;
  }
}
