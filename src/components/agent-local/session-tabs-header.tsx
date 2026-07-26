import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import type { SessionTabs } from "@/types/agent";
import "./session-tabs-header.css";

interface SessionTabsHeaderProps {
  tabs: SessionTabs | null;
  attentionTabIds?: Set<string>;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
  onRename: (tabId: string, label: string) => void;
}

export function SessionTabsHeader({
  tabs,
  attentionTabIds,
  onSelect,
  onClose,
  onRename,
}: SessionTabsHeaderProps) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  if (!tabs || tabs.tabs.length <= 1) return null;

  const startRename = (tabId: string, label: string) => {
    setEditing(tabId);
    setDraft(label);
  };
  const commitRename = () => {
    if (!editing) return;
    const label = draft.trim();
    if (label) onRename(editing, label);
    setEditing(null);
  };

  return (
    <div className="sth-tabs" role="tablist" aria-label={t("agentLocal.clone.tabs")}>
      {tabs.tabs.map((tab) => {
        const active = tab.tab_id === tabs.active_tab_id;
        const attention = !active && attentionTabIds?.has(tab.tab_id);
        const label = tab.is_main ? t("agentLocal.clone.mainTab") : tab.label;
        return (
          <div
            key={tab.tab_id}
            className={`sth-tab ${active ? "sth-tab-active" : ""} ${attention ? "sth-tab-attention" : ""}`}
            role="tab"
            aria-selected={active}
          >
            {editing === tab.tab_id ? (
              <input
                className="sth-input"
                value={draft}
                autoFocus
                onChange={(event) => setDraft(event.target.value)}
                onBlur={commitRename}
                onKeyDown={(event) => {
                  if (event.key === "Enter") commitRename();
                  if (event.key === "Escape") setEditing(null);
                }}
              />
            ) : (
              <button
                type="button"
                className="sth-label"
                onClick={() => onSelect(tab.tab_id)}
                onDoubleClick={() => startRename(tab.tab_id, label)}
              >
                {label}
              </button>
            )}
            {!tab.is_main && (
              <button
                type="button"
                className="sth-close"
                aria-label={t("agentLocal.clone.closeTab")}
                onClick={() => onClose(tab.tab_id)}
              >
                <X size="var(--icon-xs)" />
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}
