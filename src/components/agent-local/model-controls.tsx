import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAvailableModels } from "@/hooks/use-available-models";
import { admissionErrorMessage } from "@/lib/admission-error";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import { showToast } from "@/lib/toast-emitter";
import type {
  AgentSessionView,
  ContinuityCapability,
  PreserveReasoningSetting,
} from "@/types/agent-session.generated";
import i18n from "@/i18n";
import { ModelSelector } from "./model-selector";
import { ReasoningContinuitySelector } from "./reasoning-continuity-selector";
import { ReasoningSelector } from "./reasoning-selector";

interface ModelControlsProps {
  sessionId?: string;
  selectedModel: string;
  selectedProvider: string;
  onSelect: (model: string, provider: string) => void;
  reasoningMode?: string | null;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  continuityCapability?: ContinuityCapability;
  preserveReasoning?: PreserveReasoningSetting;
  onPreserveReasoningChange?: (setting: PreserveReasoningSetting) => void;
  fastModeEnabled: boolean;
  fastModePending: boolean;
  onFastModeChange: (enabled: boolean) => void;
  align?: "left" | "right";
}

export function ModelControls({
  sessionId,
  selectedModel,
  selectedProvider,
  onSelect,
  reasoningMode,
  onReasoningModeChange,
  continuityCapability,
  preserveReasoning = "off",
  onPreserveReasoningChange,
  fastModeEnabled,
  fastModePending,
  onFastModeChange,
  align = "left",
}: ModelControlsProps) {
  const [continuity, setContinuity] = useState<{
    capability?: ContinuityCapability;
    setting: PreserveReasoningSetting;
    model: string;
    provider: string;
  }>();
  const { groups } = useAvailableModels();
  const selectedEntry = useMemo(
    () => groups.get(selectedProvider)?.find((model) => model.id === selectedModel) ?? null,
    [groups, selectedModel, selectedProvider],
  );

  useEffect(() => {
    if (!sessionId) return;
    let current = true;
    void invoke<AgentSessionView>("get_agent_session", { id: sessionId }).then((session) => {
      if (!current || session.model !== selectedModel || session.provider !== selectedProvider) return;
      setContinuity({
        capability: session.continuity_capability,
        setting: session.preserve_reasoning,
        model: session.model,
        provider: session.provider,
      });
    }).catch((error: unknown) => {
      showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
    });
    return () => { current = false; };
  }, [selectedModel, selectedProvider, sessionId]);

  const changeContinuity = (setting: PreserveReasoningSetting) => {
    if (!sessionId) return;
    void invoke("update_session_continuity", { id: sessionId, setting }).then(() => {
      setContinuity((current) => current && { ...current, setting });
      onPreserveReasoningChange?.(setting);
    }).catch((error: unknown) => {
      showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
    });
  };

  return (
    <>
      <ModelSelector
        groups={groups}
        selectedModel={selectedModel}
        selectedProvider={selectedProvider}
        onSelect={onSelect}
        align={align}
      />
      <ReasoningSelector
        key={`${selectedProvider}:${selectedModel}`}
        model={selectedEntry}
        reasoningMode={reasoningMode}
        onChange={onReasoningModeChange}
        fastModeEnabled={fastModeEnabled}
        fastModePending={fastModePending}
        onFastModeChange={onFastModeChange}
        align={align}
      />
      {sessionId && continuity?.model === selectedModel && continuity.provider === selectedProvider && (
        <ReasoningContinuitySelector
          capability={continuity.capability ?? continuityCapability}
          setting={continuity.setting ?? preserveReasoning}
          onChange={changeContinuity}
        />
      )}
    </>
  );
}
