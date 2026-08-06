import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { SettingsEntryList } from "@/components/settings/shell/settings-entry-list";
import type { OllamaModel } from "@/types/agent";
import { ModelfileViewer } from "./modelfile-viewer";

interface OllamaModelfileViewProps {
  models: OllamaModel[];
  selected: string | null;
  onSelect: (name: string | null) => void;
}

export function OllamaModelfileView({ models, selected, onSelect }: OllamaModelfileViewProps) {
  const { t } = useTranslation();
  const entries = useMemo(
    () => models.map((model) => ({ id: model.name, label: model.name })),
    [models],
  );

  if (selected) {
    return (
      <ModelfileViewer
        modelName={selected}
        onBack={() => onSelect(null)}
        onDeleted={() => onSelect(null)}
      />
    );
  }

  return (
    <SettingsEntryList
      entries={entries}
      emptyMessage={t("ollama.noInstalledModels")}
      onSelect={onSelect}
    />
  );
}
