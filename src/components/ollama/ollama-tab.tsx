import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import { EmptyState } from "@/components/ui/empty-state";
import { ModelfileIcon, ModelsIcon } from "@/components/ui/model-browser-icons";
import { ollamaSetupSkippedPatch } from "@/lib/ollama-setup-gate";
import { ModelfileList } from "./modelfile-list";
import { ModelfileViewer } from "./modelfile-viewer";
import { ModelSearch } from "./model-search";
import { ModelVariantsList } from "./model-variants-list";
import { ModelProfile } from "./model-profile";
import { OllamaSetupScreen } from "./ollama-setup-screen";
import type { RegistryModel } from "@/types/agent";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import "./ollama.css";

interface OllamaTabProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

export function useOllamaTabSlots({ navState, onNavChange, onNavReplace }: OllamaTabProps): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const subTab = navState.ollamaSubTab;
  const selectedInstalled = navState.ollamaInstalledModel;
  const selectedFamily = navState.ollamaFamily;
  const selectedVariant = navState.ollamaVariant;
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

  useEffect(() => {
    if (!selectedInstalled && ollamaModels.models.length > 0) {
      onNavReplace({ ollamaInstalledModel: ollamaModels.models[0].name });
    }
  }, [ollamaModels.models, selectedInstalled, onNavReplace]);

  const setupList = useMemo(() => (
    <div className="ollama-setup-list-placeholder" />
  ), []);

  const setupDetail = useMemo(() => (
    <div className="ollama-setup-detail">
      <OllamaSetupScreen
        onComplete={async () => {
          await invoke("patch_advanced_settings", { patch: ollamaSetupSkippedPatch(false) });
          setOllamaInstalled(true);
        }}
      />
    </div>
  ), []);

  const list = useMemo(() => (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0 }}>
      <div className="ollama-subtabs">
        {(["modelfile", "models"] as const).map((tab) => (
          <button
            key={tab}
            className={`ollama-subtab ${subTab === tab ? "active" : ""}`}
            onClick={() => onNavChange({ ollamaSubTab: tab })}
          >
            {tab === "modelfile" ? (
              <><ModelfileIcon /> {t("ollama.modelfileTab")}</>
            ) : (
              <><ModelsIcon /> {t("ollama.modelsTab")}</>
            )}
          </button>
        ))}
      </div>
      {subTab === "modelfile" ? (
          <ModelfileList
            models={ollamaModels.models}
            selectedModel={selectedInstalled}
            onSelect={(model) => onNavChange({ ollamaInstalledModel: model })}
          />
      ) : selectedFamily ? (
        <ModelVariantsList
          familyName={selectedFamily}
          selectedVariant={selectedVariant}
          onSelectVariant={(variant) => onNavChange({ ollamaVariant: variant })}
          onBack={() => onNavChange({ ollamaFamily: null, ollamaVariant: null })}
        />
      ) : (
        <ModelSearch
          query={searchQuery}
          setQuery={setSearchQuery}
          results={searchResults}
          setResults={setSearchResults}
          searching={searching}
          setSearching={setSearching}
          onSelectFamily={(f) => onNavChange({ ollamaFamily: f, ollamaVariant: null })}
          selectedFamily={selectedFamily}
        />
      )}
    </div>
  ), [
    ollamaModels.models,
    onNavChange,
    searchQuery,
    searchResults,
    searching,
    selectedFamily,
    selectedInstalled,
    selectedVariant,
    subTab,
    t,
  ]);

  const detail = useMemo(() => {
    if (subTab === "modelfile" && selectedInstalled) {
      return (
        <ModelfileViewer
          modelName={selectedInstalled}
          onDeleted={() => onNavReplace({ ollamaInstalledModel: null })}
        />
      );
    }
    if (subTab === "models" && selectedFamily) {
      return (
        <ModelProfile
          familyName={selectedFamily}
          variantFullName={selectedVariant}
        />
      );
    }
    return (
      <div style={{
        flex: 1, display: "flex",
        alignItems: "center", justifyContent: "center",
      }}>
        <EmptyState message={t("ollama.selectModel")} />
      </div>
    );
  }, [onNavReplace, selectedFamily, selectedInstalled, selectedVariant, subTab, t]);

  return useMemo(
    () => (ollamaInstalled === false ? { list: setupList, detail: setupDetail } : { list, detail }),
    [detail, list, ollamaInstalled, setupDetail, setupList],
  );
}
