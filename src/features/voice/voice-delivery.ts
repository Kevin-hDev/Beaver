import {
  acknowledgeVoiceDelivery,
  applyVoiceDelivery,
} from "@/hooks/composer-draft-store";
import type { VoiceDeliverySnapshot, VoiceSnapshot } from "@/types/voice.generated";
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
  const outcome = localOutcome === "destination-closed" ? "closed" : localOutcome;
  const snapshot = await sendAcknowledgement(delivery, outcome);
  acknowledgeVoiceDelivery(delivery.draftKey, delivery.id);
  return snapshot;
}
