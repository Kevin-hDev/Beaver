import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import { useOllamaModels } from "@/hooks/use-ollama-models";
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
  selectedFamily: string | null;
}

export function ModelSearch({
  query, setQuery, results, setResults,
  searching, setSearching,
  onSelectFamily, selectedFamily,
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

  const isFamilyInstalled = (familyName: string): boolean => {
    return localModels.some((m) => m.name.startsWith(`${familyName}:`));
  };

  return (
    <div className="msearch-root">
      <div className="msearch-bar">
        <input
          className="ollama-search-input"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => { if (e.code === "Enter") void handleSearch(); }}
          placeholder={t("ollama.searchPlaceholder")}
        />
      </div>
      <div className="msearch-results">
        {searching && (
          <div className="msearch-loading">
            {t("history.loading")}
          </div>
        )}
        {!searching && results.length === 0 && query.trim() === "" && (
          <div className="msearch-hint">
            {t("ollama.searchHint")}
          </div>
        )}
        {results.map((m) => {
          const installed = isFamilyInstalled(m.name);
          const isActive = selectedFamily ? selectedFamily === m.name : false;
          return (
            <div
              key={m.name}
              className={`ollama-model-item msearch-item-row ${isActive ? "active" : ""}`}
              role="button"
              tabIndex={0}
              onClick={() => onSelectFamily(m.name)}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onSelectFamily(m.name); }}
            >
              <div className="msearch-item-content">
                <div className="msearch-item-name">
                  {m.name}
                </div>
                {m.description && (
                  <div className="msearch-item-desc">
                    {m.description}
                  </div>
                )}
              </div>
              {installed && <Check size="var(--icon-sm)" className="msearch-installed-icon" />}
            </div>
          );
        })}
      </div>
    </div>
  );
}
