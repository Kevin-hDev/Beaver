import type { VoicePhase } from "@/types/voice.generated";
import type { VoiceSnapshot } from "@/types/voice.generated";
import { dispatchVoiceAction } from "./voice-client";

export type VoiceKeyboardDecision = "none" | "consume" | "validate" | "cancel-insertion" | "toggle";

interface VoiceKeyboardInput {
  key: string;
  shiftKey: boolean;
  composing: boolean;
  origin: boolean;
  phase: VoicePhase;
}

export function decideVoiceKeyboard(input: VoiceKeyboardInput): VoiceKeyboardDecision {
  if (!input.origin || input.composing || input.shiftKey) return "none";
  if (input.key === "Enter") {
    if (input.phase === "listening") return "validate";
    if (["preparing", "transcribing", "recovering", "delivering"].includes(input.phase)) return "consume";
  }
  if (input.key === "Escape" && input.phase !== "idle") return "cancel-insertion";
  return "none";
}

export function handleVoiceKeyboard(event: KeyboardEvent, snapshot: VoiceSnapshot | null, draftKey: string): boolean {
  const destination = snapshot?.operation?.destination;
  const decision = decideVoiceKeyboard({
    key: event.key,
    shiftKey: event.shiftKey,
    composing: event.isComposing,
    origin: destination?.kind === "draft" && destination.draft_key === draftKey,
    phase: snapshot?.phase ?? "idle",
  });
  if (decision === "none") return false;
  event.preventDefault();
  event.stopPropagation();
  const operationId = snapshot?.operation?.id ?? snapshot?.delivery?.id;
  if (operationId && decision === "validate") void dispatchVoiceAction({ action: "validate", operation_id: operationId });
  if (operationId && decision === "cancel-insertion") void dispatchVoiceAction({ action: "cancel-insertion", operation_id: operationId });
  return true;
}

export function voiceShortcutValue(event: Pick<KeyboardEvent, "altKey" | "code" | "ctrlKey" | "metaKey" | "shiftKey">): string | null {
  if (!event.code || /^(Meta|Control|Alt|Shift)(Left|Right)$/.test(event.code) || (!event.metaKey && !event.ctrlKey && !event.altKey)) return null;
  return [event.metaKey ? "Meta" : "", event.ctrlKey ? "Control" : "", event.altKey ? "Alt" : "", event.shiftKey ? "Shift" : "", event.code]
    .filter(Boolean).join("+");
}

export function matchesVoiceShortcut(event: KeyboardEvent, configured: string | null): boolean {
  return configured ? voiceShortcutValue(event) === configured : false;
}
