import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { LlmFamilyGrid } from "./llm-family-grid";
import { LlmModelList } from "./llm-model-list";
import { LlmModelDetail } from "./llm-model-detail";
import { SettingsCard } from "./settings-card";
import type { RegistryModelInfo, FamilyGroup } from "./llm-types";
import type { LlmNavState } from "@/types/navigation";
import "./llm-explorer.css";

type View =
  | { kind: "idle" }
  | { kind: "search"; query: string; results: RegistryModelInfo[] }
  | { kind: "family-models"; family: string; models: RegistryModelInfo[] }
  | { kind: "detail"; model: RegistryModelInfo; prev: View };

function viewIs(v: View, k: string): boolean {
  return v.kind.startsWith(k);
}

interface LlmExplorerProps {
  navState: LlmNavState;
  onNavChange: (state: LlmNavState) => void;
}

export function LlmExplorer({ navState, onNavChange }: LlmExplorerProps) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [view, setView] = useState<View>({ kind: "idle" });
  const [families, setFamilies] = useState<FamilyGroup[]>([]);
  const [showFamilies, setShowFamilies] = useState(false);

  useEffect(() => {
    invoke<FamilyGroup[]>("list_registry_families")
      .then(setFamilies)
      .catch(() => {});
  }, [onNavChange]);

  useEffect(() => {
    let active = true;
    async function load() {
      if (navState.kind === "idle") {
        setView({ kind: "idle" });
        setShowFamilies(navState.showFamilies);
      } else if (navState.kind === "search") {
        const results = await invoke<RegistryModelInfo[]>("search_registry", { query: navState.query }).catch(() => []);
        if (active) { setView({ kind: "search", query: navState.query, results }); setShowFamilies(false); }
      } else if (navState.kind === "family") {
        const models = await invoke<RegistryModelInfo[]>("list_family_models", { family: navState.family }).catch(() => []);
        if (active) { setView({ kind: "family-models", family: navState.family, models }); setShowFamilies(false); }
      } else {
        const results = await invoke<RegistryModelInfo[]>("search_registry", { query: navState.modelKey }).catch(() => []);
        const model = results.find((item) => item.key === navState.modelKey) ?? results[0];
        if (active && model) setView({ kind: "detail", model, prev: { kind: "idle" } });
        if (active) setShowFamilies(false);
      }
    }
    void load();
    return () => { active = false; };
  }, [navState]);

  const handleSearch = useCallback(async () => {
    if (!query.trim()) return;
    try {
      const results = await invoke<RegistryModelInfo[]>("search_registry", { query: query.trim() });
      setView({ kind: "search", query: query.trim(), results });
      setShowFamilies(false);
      onNavChange({ kind: "search", query: query.trim() });
    } catch (e) {
      console.warn("[llm] search error:", e);
    }
  }, [query, onNavChange]);

  const handleFamilyClick = useCallback(async (family: string) => {
    try {
      const models = await invoke<RegistryModelInfo[]>("list_family_models", { family });
      setView({ kind: "family-models", family, models });
      setShowFamilies(false);
      onNavChange({ kind: "family", family });
    } catch (e) {
      console.warn("[llm] family error:", e);
    }
  }, [onNavChange]);

  const handleModelClick = useCallback((model: RegistryModelInfo) => {
    setView((prev) => ({ kind: "detail", model, prev }));
    const parent = navState.kind === "detail" ? navState.parent : navState;
    onNavChange({ kind: "detail", modelKey: model.key, parent });
  }, [navState, onNavChange]);

  const handleBack = useCallback(() => {
    if (viewIs(view, "detail")) {
      if (navState.kind === "detail") onNavChange(navState.parent);
    } else {
      onNavChange({ kind: "idle", showFamilies: true });
    }
  }, [view, navState, onNavChange]);

  const toggleFamilies = useCallback(() => {
    onNavChange({ kind: "idle", showFamilies: !showFamilies });
  }, [showFamilies, onNavChange]);

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 700, width: "100%", margin: "0 auto" }}>
        <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", marginBottom: 20 }}>
          LLM
        </h2>

        <SettingsCard>
          <div className="llm-search-bar">
            <input
              className="llm-search-input"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => { if (e.key.startsWith("Ent")) void handleSearch(); }}
              placeholder={t("settings.llm.searchPlaceholder")}
            />
            <button
              className={`llm-family-btn ${showFamilies ? "active" : ""}`}
              onClick={toggleFamilies}
            >
              {t("settings.llm.families")}
            </button>
          </div>
        </SettingsCard>

        {showFamilies && viewIs(view, "idle") && (
          <SettingsCard>
            <LlmFamilyGrid families={families} onSelect={(f) => void handleFamilyClick(f)} />
          </SettingsCard>
        )}

        {viewIs(view, "search") && (
          <LlmModelList
            models={(view as View & { kind: "search" }).results}
            title={(view as View & { kind: "search" }).query}
            onSelect={handleModelClick}
          />
        )}

        {viewIs(view, "family") && (
          <LlmModelList
            models={(view as View & { kind: "family-models" }).models}
            title={(view as View & { kind: "family-models" }).family}
            onSelect={handleModelClick}
            onBack={() => onNavChange({ kind: "idle", showFamilies: true })}
          />
        )}

        {viewIs(view, "detail") && (
          <LlmModelDetail
            model={(view as View & { kind: "detail" }).model}
            onBack={handleBack}
          />
        )}
      </div>
    </div>
  );
}
