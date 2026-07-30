import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import { ModelfileIcon, ModelsIcon } from "@/components/ui/model-browser-icons";
import { SettingsPanel } from "@/components/settings/shell/settings-panel";
import { SettingsTabbar } from "@/components/settings/shell/settings-tabbar";
import { ollamaSetupSkippedPatch } from "@/lib/ollama-setup-gate";
import { OllamaSetupScreen } from "./ollama-setup-screen";
import { OllamaModelfileView } from "./ollama-modelfile-view";
import { OllamaModelsView } from "./ollama-models-view";
import type { RegistryModel } from "@/types/agent";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import "./ollama.css";

interface OllamaTabProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

export function useOllamaTabContent({ navState, onNavChange, onNavReplace }: OllamaTabProps): React.ReactNode {
  const { t } = useTranslation();
  const subTab = navState.ollamaSubTab;
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<RegistryModel[]>([]);
  const [searching, setSearching] = useState(false);
  const [ollamaInstalled, setOllamaInstalled] = useState<boolean | null>(null);
  const ollamaModels = useOllamaModels({ enabled: ollamaInstalled === true });

  useEffect(() => {
    invoke<boolean>("is_ollama_installed")
      .then(setOllamaInstalled)
      .catch(() => setOllamaInstalled(true));
  }, []);

  const tabs = useMemo(() => [
    { id: "modelfile" as const, label: t("ollama.modelfileTab"), icon: <ModelfileIcon size="var(--icon-md)" /> },
    { id: "models" as const, label: t("ollama.modelsTab"), icon: <ModelsIcon size="var(--icon-md)" /> },
  ], [t]);

  const search = useMemo(() => ({
    query: searchQuery,
    setQuery: setSearchQuery,
    results: searchResults,
    setResults: setSearchResults,
    searching,
    setSearching,
  }), [searchQuery, searchResults, searching]);

  const detail = useMemo(() => {
    if (ollamaInstalled === false) {
      return (
        <div className="ollama-setup-detail">
          <OllamaSetupScreen
            onComplete={async () => {
              await invoke("patch_advanced_settings", { patch: ollamaSetupSkippedPatch(false) });
              setOllamaInstalled(true);
            }}
          />
        </div>
      );
    }
    return (
      <SettingsPanel title={t("settings.tabs.ollama")}>
        <SettingsTabbar
          items={tabs}
          active={subTab}
          label={t("settings.tabs.ollama")}
          onChange={(ollamaSubTab) => onNavChange({ ollamaSubTab })}
        />
        {subTab === "modelfile" ? (
          <OllamaModelfileView
            models={ollamaModels.models}
            selected={navState.ollamaInstalledModel}
            onSelect={(ollamaInstalledModel) => onNavReplace({ ollamaInstalledModel })}
          />
        ) : (
          <OllamaModelsView
            search={search}
            family={navState.ollamaFamily}
            variant={navState.ollamaVariant}
            onSelectFamily={(ollamaFamily) => onNavReplace({ ollamaFamily, ollamaVariant: null })}
            onSelectVariant={(ollamaVariant) => onNavReplace({ ollamaVariant })}
          />
        )}
      </SettingsPanel>
    );
  }, [
    navState.ollamaFamily,
    navState.ollamaInstalledModel,
    navState.ollamaVariant,
    ollamaInstalled,
    ollamaModels.models,
    onNavChange,
    onNavReplace,
    search,
    subTab,
    t,
    tabs,
  ]);

  return detail;
}
