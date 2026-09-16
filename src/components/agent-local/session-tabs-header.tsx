import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import type { SessionTabs } from "@/types/agent";
import { useVoiceSnapshot } from "@/features/voice/voice-store";
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
  const voiceSnapshot = useVoiceSnapshot();
  const voiceDestination = voiceSnapshot?.operation?.destination;
  const voiceSessionId = voiceDestination?.kind === "draft" && voiceDestination.draft_key.startsWith("session:")
    ? voiceDestination.draft_key.slice("session:".length) : null;
  useEffect(() => {
    const goToVoice = (event: Event) => {
      const draftKey = (event as CustomEvent<unknown>).detail;
      if (typeof draftKey !== "string" || !draftKey.startsWith("session:")) return;
      const sessionId = draftKey.slice("session:".length);
      const tab = tabs?.tabs.find((item) => item.session_id === sessionId);
      if (tab) onSelect(tab.tab_id);
    };
    window.addEventListener("beaver:voice-go-to-draft", goToVoice);
    return () => window.removeEventListener("beaver:voice-go-to-draft", goToVoice);
  }, [onSelect, tabs]);
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
                className="field sth-input"
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
            {voiceSessionId === tab.session_id && <span className="sth-voice" aria-label={t("voice.tabIndicator")}>●</span>}
          </div>
        );
      })}
    </div>
  );
}
