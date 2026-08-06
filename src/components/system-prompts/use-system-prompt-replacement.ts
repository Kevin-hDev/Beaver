import { useState } from "react";
import type { SystemPromptView } from "@/types/system-prompts";

export type PromptReplacementDestination = "beaver" | "ollama";

interface PendingReplacement {
  content: string;
  destination: PromptReplacementDestination;
  selectionKey: string;
}

interface UseSystemPromptReplacementOptions {
  view: SystemPromptView | null;
  selectionKey: string;
  replacePrompt: (destination: PromptReplacementDestination) => Promise<void>;
}

export function useSystemPromptReplacement({
  view,
  selectionKey,
  replacePrompt,
}: UseSystemPromptReplacementOptions) {
  const [pendingReplacement, setPendingReplacement] = useState<PendingReplacement | null>(null);

  const requestReplacement = (destination: PromptReplacementDestination) => {
    if (view?.selection === "custom" && view.content.length > 0) {
      setPendingReplacement({ content: view.content, destination, selectionKey });
      return;
    }
    void replacePrompt(destination);
  };

  const cancelReplacement = () => setPendingReplacement(null);

  const confirmReplacement = () => {
    const pending = pendingReplacement;
    setPendingReplacement(null);
    if (!pending || pending.selectionKey !== selectionKey
      || view?.selection !== "custom" || view.content !== pending.content) return;
    void replacePrompt(pending.destination);
  };

  return {
    pendingReplacement: pendingReplacement?.selectionKey === selectionKey
      && view?.selection === "custom"
      && view.content === pendingReplacement.content
      ? pendingReplacement
      : null,
    requestReplacement,
    cancelReplacement,
    confirmReplacement,
  };
}
