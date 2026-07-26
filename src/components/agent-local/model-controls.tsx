import { useMemo } from "react";
import { useAvailableModels } from "@/hooks/use-available-models";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import { ModelSelector } from "./model-selector";
import { ReasoningSelector } from "./reasoning-selector";

interface ModelControlsProps {
  selectedModel: string;
  selectedProvider: string;
  onSelect: (model: string, provider: string) => void;
  reasoningMode?: string | null;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  align?: "left" | "right";
}

export function ModelControls({
  selectedModel,
  selectedProvider,
  onSelect,
  reasoningMode,
  onReasoningModeChange,
  align = "left",
}: ModelControlsProps) {
  const { groups } = useAvailableModels();
  const selectedEntry = useMemo(
    () => groups.get(selectedProvider)?.find((model) => model.id === selectedModel) ?? null,
    [groups, selectedModel, selectedProvider],
  );

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
        align={align}
      />
    </>
  );
}
