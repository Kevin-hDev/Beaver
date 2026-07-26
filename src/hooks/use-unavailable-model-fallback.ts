import { useEffect, type Dispatch, type SetStateAction } from "react";
import type { AvailableModel } from "@/hooks/use-available-models";
import { legacyXaiReplacement } from "@/lib/model-migrations";
import { reasoningModeOptions } from "@/lib/reasoning-modes";

interface SelectedModel {
  model: string;
  provider: string;
}

type UpdateModel = (
  id: string,
  model: string,
  provider?: string,
  reasoningMode?: string | null,
  supportsThinking?: boolean,
) => Promise<void>;

interface Options {
  availableModels: Map<string, AvailableModel[]>;
  model: string;
  provider: string;
  reasoningMode?: string | null;
  activeSessionId?: string | null;
  updateModel: UpdateModel;
  setWelcomeModel: Dispatch<SetStateAction<SelectedModel | null>>;
}

export function useUnavailableModelFallback({
  availableModels,
  model,
  provider,
  reasoningMode,
  activeSessionId,
  updateModel,
  setWelcomeModel,
}: Options) {
  useEffect(() => {
    if (!model || availableModels.size === 0) return;
    const providerModels = availableModels.get(provider);
    if (!providerModels || providerModels.some((entry) => entry.id === model)) return;

    const initialReplacement = legacyXaiReplacement(provider, model, reasoningMode, []);
    const replacementModel = initialReplacement
      ? providerModels.find((entry) => entry.id === initialReplacement.model)
      : null;
    if (replacementModel && activeSessionId) {
      const supportedModes = reasoningModeOptions(replacementModel).map((option) => option.mode);
      const replacement = legacyXaiReplacement(provider, model, reasoningMode, supportedModes);
      if (replacement) {
        void updateModel(
          activeSessionId,
          replacement.model,
          provider,
          replacement.reasoningMode,
          replacementModel.supports_thinking ?? false,
        );
        return;
      }
    }

    const fallback = Array.from(availableModels.values()).flat()[0];
    if (!fallback) return;
    if (activeSessionId) {
      void updateModel(activeSessionId, fallback.id, fallback.provider_id);
    } else {
      setWelcomeModel({ model: fallback.id, provider: fallback.provider_id });
    }
  }, [
    activeSessionId,
    availableModels,
    model,
    provider,
    reasoningMode,
    setWelcomeModel,
    updateModel,
  ]);
}
