import { useState, useRef, useCallback } from "react";

type SwitchChoice = "new" | "continue";

interface PendingSwitch {
  model: string;
  provider: string;
}

interface UseModelSwitchParams {
  currentModel: string;
  currentProvider: string;
  messagesLength: number;
  onApplySwitch?: (model: string, provider: string) => void;
  onNewSession?: (model: string, provider: string) => void;
}

interface UseModelSwitchReturn {
  pendingSwitch: PendingSwitch | null;
  setPendingSwitch: React.Dispatch<React.SetStateAction<PendingSwitch | null>>;
  handleModelSelect: (newModel: string, newProvider: string) => void;
  rememberedRef: React.MutableRefObject<SwitchChoice | null>;
}

export function useModelSwitch({
  currentModel,
  currentProvider,
  messagesLength,
  onApplySwitch,
  onNewSession,
}: UseModelSwitchParams): UseModelSwitchReturn {
  const [pendingSwitch, setPendingSwitch] = useState<PendingSwitch | null>(null);
  const rememberedRef = useRef<SwitchChoice | null>(null);

  const handleModelSelect = useCallback(
    (newModel: string, newProvider: string) => {
      if (newModel === currentModel && newProvider === currentProvider) return;
      const hasMessages = messagesLength > 0;
      if (!hasMessages) {
        onApplySwitch?.(newModel, newProvider);
        return;
      }
      if (rememberedRef.current === "continue") {
        onApplySwitch?.(newModel, newProvider);
        return;
      }
      if (rememberedRef.current === "new") {
        onNewSession?.(newModel, newProvider);
        return;
      }
      setPendingSwitch({ model: newModel, provider: newProvider });
    },
    [currentModel, currentProvider, messagesLength, onApplySwitch, onNewSession],
  );

  return { pendingSwitch, setPendingSwitch, handleModelSelect, rememberedRef };
}
