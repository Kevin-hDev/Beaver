import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { X, MagnifyingGlass } from "@/components/ui/icons";
import type { ProviderCategory, ProviderSpec } from "@/types/api";
import { ConnectorCard } from "./connector-card";

type CategoryTab = "all" | ProviderCategory;

const TABS: { id: CategoryTab; i18nKey: string }[] = [
  { id: "all", i18nKey: "apiKeys.connectors.tabs.all" },
  { id: "llm", i18nKey: "apiKeys.connectors.tabs.llm" },
  { id: "search", i18nKey: "apiKeys.connectors.tabs.search" },
  { id: "scraping", i18nKey: "apiKeys.connectors.tabs.scraping" },
  { id: "forecast", i18nKey: "apiKeys.connectors.tabs.forecast" },
];

interface ConnectorsModalProps {
  catalog: ProviderSpec[];
  configuredIds: string[];
  onPick: (provider: ProviderSpec) => void;
  onClose: () => void;
}

export function ConnectorsModal({
  catalog,
  configuredIds,
  onPick,
  onClose,
}: ConnectorsModalProps) {
  const { t } = useTranslation();
  const [category, setCategory] = useState<CategoryTab>("all");
  const [query, setQuery] = useState("");

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return catalog.filter((p) => {
      if (category !== "all" && p.category !== category) return false;
      if (!q) return true;
      return (
        p.display_name.toLowerCase().includes(q) ||
        p.short_description.toLowerCase().includes(q) ||
        p.short_description_en.toLowerCase().includes(q) ||
        p.id.toLowerCase().includes(q)
      );
    });
  }, [catalog, category, query]);

  return (
    <div className="wk-dialog-overlay" role="button" tabIndex={-1} aria-label="Close dialog" onClick={onClose} onKeyDown={(e) => { if (e.key === "Escape") onClose(); }}>
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div
        className="ak-connectors-modal"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
      >
        <header className="ak-connectors-header">
          <div className="ak-connectors-heading">
            <div className="ak-connectors-title">
              {t("apiKeys.connectors.title")}
            </div>
            <div className="ak-connectors-subtitle">
              {t("apiKeys.connectors.subtitle")}
            </div>
          </div>
          <button type="button" className="wk-dialog-close" onClick={onClose}>
            <X size="var(--icon-md)" />
          </button>
        </header>

        <div className="ak-connectors-tabs">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              className={`ak-connectors-tab ${category === tab.id ? "active" : ""}`}
              onClick={() => setCategory(tab.id)}
            >
              {t(tab.i18nKey)}
            </button>
          ))}
        </div>

        <div className="ak-connectors-search">
          <MagnifyingGlass
            size="var(--icon-md)"
            className="ak-connectors-search-icon"
            weight="regular"
          />
          <input
            type="text"
            className="ak-connectors-search-input"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("apiKeys.connectors.search")}
            autoFocus
          />
        </div>

        <div className="ak-connectors-grid">
          {filtered.length === 0 ? (
            <div className="ak-connectors-empty">
              {t("apiKeys.connectors.empty")}
            </div>
          ) : (
            filtered.map((p) => (
              <ConnectorCard
                key={p.id}
                provider={p}
                configured={configuredIds.includes(p.id)}
                onAdd={() => onPick(p)}
              />
            ))
          )}
        </div>
      </div>
    </div>
  );
}
