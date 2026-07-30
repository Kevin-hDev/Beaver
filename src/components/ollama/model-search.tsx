import { useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import { SettingsEntryList } from "@/components/settings/shell/settings-entry-list";
import type { RegistryModel } from "@/types/agent";
import "./ollama.css";
import "./ollama-details.css";
import "./model-search.css";

interface ModelSearchProps {
  query: string;
  setQuery: (q: string) => void;
  results: RegistryModel[];
  setResults: (list: RegistryModel[]) => void;
  searching: boolean;
  setSearching: (b: boolean) => void;
  onSelectFamily: (familyName: string) => void;
}

export function ModelSearch({
  query, setQuery, results, setResults,
  searching, setSearching, onSelectFamily,
}: ModelSearchProps) {
  const { t } = useTranslation();
  const { models: localModels } = useOllamaModels();

  const handleSearch = useCallback(async () => {
    if (!query.trim()) return;
    setSearching(true);
    try {
      const list = await invoke<RegistryModel[]>("search_ollama_models", { query: query.trim() });
      setResults(list);
    } catch (e: unknown) {
      console.warn("[ollama] search:", e);
      setResults([]);
    } finally {
      setSearching(false);
    }
  }, [query, setSearching, setResults]);

  const entries = useMemo(
    () => results.map((model) => ({
      id: model.name,
      label: model.name,
      description: model.description ?? undefined,
      trailing: localModels.some((local) => local.name.startsWith(`${model.name}:`))
        ? <Check size="var(--icon-sm)" className="msearch-installed-icon" />
        : undefined,
    })),
    [localModels, results],
  );

  return (
    <>
      <div className="msearch-bar">
        <input
          className="ollama-search-input"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => { if (e.code === "Enter") void handleSearch(); }}
          placeholder={t("ollama.searchPlaceholder")}
        />
      </div>
      {searching ? (
        <p className="settings-panel-description">{t("history.loading")}</p>
      ) : (
        <SettingsEntryList
          entries={entries}
          emptyMessage={t("ollama.searchHint")}
          onSelect={onSelectFamily}
        />
      )}
    </>
  );
}
