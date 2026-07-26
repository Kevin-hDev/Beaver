import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { X, MagnifyingGlass } from "@/components/ui/icons";
import { getMcpDescription } from "@/types/mcp";
import type { McpCategory, McpConnectorSpec } from "@/types/mcp";
import { McpBrowseCard } from "./mcp-browse-card";
import { EmptyState } from "@/components/ui/empty-state";
import "./mcp-browse-modal.css";

type BrowseTab = "mcp" | "plugins";
type CategoryTab = "all" | McpCategory;

const CATEGORIES: { id: CategoryTab; i18nKey: string }[] = [
  { id: "all", i18nKey: "connectors.browse.categories.all" },
  { id: "productivity", i18nKey: "connectors.browse.categories.productivity" },
  { id: "design", i18nKey: "connectors.browse.categories.design" },
  { id: "devtools", i18nKey: "connectors.browse.categories.devtools" },
  { id: "communication", i18nKey: "connectors.browse.categories.communication" },
  { id: "ai-ml", i18nKey: "connectors.browse.categories.ai-ml" },
  { id: "scraping", i18nKey: "connectors.browse.categories.scraping" },
  { id: "community", i18nKey: "connectors.browse.categories.community" },
];

interface McpBrowseModalProps {
  catalog: McpConnectorSpec[];
  configuredIds: string[];
  onPick: (connector: McpConnectorSpec) => void;
  onClose: () => void;
}

export function McpBrowseModal({ catalog, configuredIds, onPick, onClose }: McpBrowseModalProps) {
  const { t, i18n } = useTranslation();
  const [browseTab, setBrowseTab] = useState<BrowseTab>("mcp");
  const [category, setCategory] = useState<CategoryTab>("all");
  const [query, setQuery] = useState("");

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const filtered = useMemo(() => {
    const q = query.toLowerCase();
    return catalog.filter((c) => {
      if (category !== "all" && c.category !== category) return false;
      if (!q) return true;
      const desc = getMcpDescription(c, i18n.language);
      return c.display_name.toLowerCase().includes(q) || desc.toLowerCase().includes(q) || c.author.toLowerCase().includes(q);
    });
  }, [catalog, category, query, i18n.language]);

  return (
    <div className="wk-dialog-overlay" role="button" tabIndex={-1} aria-label="Close dialog" onClick={onClose} onKeyDown={(e) => { if (e.key === "Escape") onClose(); }}>
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions -- dialog stop-propagation pattern */}
      <div className="mcb-modal" onClick={(e) => e.stopPropagation()} role="dialog">
        <header className="mcb-header">
          <div>
            <div className="mcb-title">{t("connectors.browse.title")}</div>
            <div className="mcb-subtitle">{t("connectors.browse.subtitle")}</div>
          </div>
          <button type="button" className="wk-dialog-close" onClick={onClose}><X size="var(--icon-md)" /></button>
        </header>

        <div className="mcb-top-tabs">
          <button type="button" className={`mcb-top-tab ${browseTab === "mcp" ? "active" : ""}`} onClick={() => setBrowseTab("mcp")}>
            {t("connectors.browse.tabMcp")}
          </button>
          <button type="button" className={`mcb-top-tab ${browseTab === "plugins" ? "active" : ""}`} onClick={() => setBrowseTab("plugins")}>
            {t("connectors.browse.tabPlugins")}
          </button>
        </div>

        {browseTab === "plugins" ? (
          <div className="mcb-plugins-empty">
            <EmptyState message={t("connectors.browse.pluginsEmpty")} />
          </div>
        ) : (
          <>
            <div className="mcb-categories">
              {CATEGORIES.map((cat) => (
                <button key={cat.id} type="button" className={`ak-connectors-tab ${category === cat.id ? "active" : ""}`} onClick={() => setCategory(cat.id)}>
                  {t(cat.i18nKey)}
                </button>
              ))}
            </div>

            <div className="ak-connectors-search">
              <MagnifyingGlass size="var(--icon-md)" className="ak-connectors-search-icon" weight="regular" />
              <input type="text" className="ak-connectors-search-input" value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("connectors.browse.search")} autoFocus />
            </div>

            <div className="ak-connectors-grid">
              {filtered.length === 0 ? (
                <div className="ak-connectors-empty">{t("connectors.browse.empty")}</div>
              ) : (
                filtered.map((c) => (
                  <McpBrowseCard key={c.id} connector={c} configured={configuredIds.includes(c.id)} onAdd={() => onPick(c)} />
                ))
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
