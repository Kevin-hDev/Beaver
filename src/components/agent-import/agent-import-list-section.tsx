import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { MagnifyingGlass } from "@/components/ui/icons";
import { allItemIds } from "@/lib/agent-import-selection";
import type { AgentImportItem } from "@/types/agent-import";

interface AgentImportListSectionProps {
  title: string;
  items: AgentImportItem[];
  selectedIds: Set<string>;
  onChange: (ids: Set<string>) => void;
  lockedIds?: Set<string>;
  searchable?: boolean;
  bulkActions?: boolean;
}

export function AgentImportListSection({
  title,
  items,
  selectedIds,
  onChange,
  lockedIds = new Set(),
  searchable = false,
  bulkActions = false,
}: AgentImportListSectionProps) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const visibleItems = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) return items;
    return items.filter((item) =>
      `${item.name} ${item.description}`.toLowerCase().includes(normalized),
    );
  }, [items, query]);
  const availableIds = useMemo(() => allItemIds(items), [items]);

  if (items.length === 0) return null;

  return (
    <section className="aim-item-section">
      <div className="aim-section-header">
        <div>
          <h3 className="aim-section-title">{title}</h3>
          <span className="aim-section-count">
            {t("agentImport.detail.selectedCount", {
              selected: selectedIds.size,
              total: items.length,
            })}
          </span>
        </div>
        {bulkActions && (
          <div className="aim-bulk-actions">
            <button
              type="button"
              onClick={() => onChange(availableIds)}
              disabled={selectedIds.size === availableIds.size}
            >
              {t("agentImport.actions.all")}
            </button>
            <button
              type="button"
              onClick={() => onChange(new Set())}
              disabled={selectedIds.size === 0}
            >
              {t("agentImport.actions.none")}
            </button>
          </div>
        )}
      </div>

      {searchable && (
        <label className="aim-search">
          <MagnifyingGlass size="var(--icon-sm)" />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t("agentImport.detail.searchSkills")}
          />
        </label>
      )}

      <div className="aim-item-list">
        {visibleItems.map((item) => {
          const locked = lockedIds.has(item.id);
          return (
            <label className="aim-item" key={item.id}>
              <input
                type="checkbox"
                checked={selectedIds.has(item.id)}
                disabled={!item.available || locked}
                onChange={() => {
                  const next = new Set(selectedIds);
                  if (next.has(item.id)) next.delete(item.id);
                  else next.add(item.id);
                  onChange(next);
                }}
              />
              <span className="aim-item-copy">
                <span className="aim-item-name">{item.name}</span>
                {item.description && (
                  <span className="aim-item-description">{item.description}</span>
                )}
                {item.updateAvailable && (
                  <span className="aim-item-update">
                    {t("agentImport.detail.updateAvailable")}
                  </span>
                )}
                {locked && (
                  <span className="aim-item-imported">
                    {t("agentImport.detail.imported")}
                  </span>
                )}
              </span>
            </label>
          );
        })}
      </div>
    </section>
  );
}
