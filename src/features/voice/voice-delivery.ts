import {
  acknowledgeVoiceDelivery,
  applyVoiceDelivery,
} from "@/hooks/composer-draft-store";
import type { VoiceDeliverySnapshot, VoiceSnapshot } from "@/types/voice.generated";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";
import { dispatchVoiceAction } from "./voice-client";

type Acknowledge = (delivery: VoiceDeliverySnapshot, outcome: "inserted" | "already-inserted" | "closed") => Promise<VoiceSnapshot>;

const acknowledge: Acknowledge = (delivery, outcome) => dispatchVoiceAction({
  action: "acknowledge-delivery",
  result_id: delivery.id,
  outcome,
});

export async function deliverVoiceResult(
  delivery: VoiceDeliverySnapshot,
  sendAcknowledgement: Acknowledge = acknowledge,
): Promise<VoiceSnapshot> {
  const localOutcome = applyVoiceDelivery(delivery);
  if (localOutcome === "inserted" && delivery.microphoneDisconnected) {
    showToast(i18n.t("voice.notice.microphoneDisconnected"), "info");
  }
  const outcome = localOutcome === "destination-closed" ? "closed" : localOutcome;
  const snapshot = await sendAcknowledgement(delivery, outcome);
  acknowledgeVoiceDelivery(delivery.draftKey, delivery.id);
  return snapshot;
}
