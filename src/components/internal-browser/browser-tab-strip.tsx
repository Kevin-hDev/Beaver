import { useId, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Plus, X } from "@/components/ui/icons";
import { ALT_LABEL } from "@/lib/platform";
import { BrowserTabFavicon } from "./browser-tab-favicon";
import type { BrowserTabState } from "./browser-types";
import { useBrowserTabReorder } from "./use-browser-tab-reorder";

interface BrowserTabStripProps {
  tabs: BrowserTabState[];
  favicons?: ReadonlyMap<string, string>;
  activeTabId: string;
  enabled: boolean;
  conversationId: string;
  onReorder: (tabIds: string[]) => Promise<boolean>;
  onReorderError: () => void;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
  onAdd: () => void;
}

export function BrowserTabStrip(props: BrowserTabStripProps) {
  const { t } = useTranslation();
  const containerRef = useRef<HTMLDivElement>(null);
  const hintId = useId();
  const tabReorder = useBrowserTabReorder({
    tabs: props.tabs,
    activeTabId: props.activeTabId,
    conversationId: props.conversationId,
    enabled: props.enabled,
    onReorder: props.onReorder,
    onReorderError: props.onReorderError,
    containerRef,
  });
  const tabsById = new Map(props.tabs.map((tab) => [tab.id, tab]));

  return (
    <div className="ib-tabs-bar">
      <div
        ref={containerRef}
        className="ib-tabs-scroll"
        role="tablist"
        aria-label={t("browser.tabsLabel")}
        aria-describedby={hintId}
        aria-busy={tabReorder.pending || undefined}
      >
        <span id={hintId} className="sr-only">{t("browser.reorderTabsHint", { alt: ALT_LABEL })}</span>
        {tabReorder.drag.order.map((id) => {
          const tab = tabsById.get(id);
          if (!tab) return null;
          const active = tab.id === props.activeTabId;
          const title = tab.title || t("browser.newTab");
          return (
            <div
              className={`ib-tab ${active ? "ib-tab-active" : ""}`}
              key={tab.id}
              {...tabReorder.drag.itemProps(tab.id)}
            >
              <button
                className="ib-tab-select"
                type="button"
                role="tab"
                aria-selected={active}
                data-browser-tab-id={tab.id}
                title={title}
                onPointerDown={(event) => {
                  if (event.button === 0) event.preventDefault();
                  tabReorder.handlePointerDown(tab.id, event);
                }}
                onKeyDown={(event) => tabReorder.handleKeyDown(tab.id, event)}
                onClick={(event) => {
                  // Les clics clavier n'ont pas de pointerdown pour effacer didDrag.
                  if (event.detail === 0 || !tabReorder.drag.didDrag()) {
                    event.currentTarget.focus();
                    props.onSelect(tab.id);
                  }
                }}
              >
                <BrowserTabFavicon pngBase64={props.favicons?.get(tab.id)} />
                <span className="ib-tab-title">{title}</span>
              </button>
              <button
                className="ib-tab-close"
                type="button"
                aria-label={t("browser.closeTab", { title })}
                onClick={() => props.onClose(tab.id)}
              >
                <X size="var(--icon-xs)" aria-hidden="true" />
              </button>
            </div>
          );
        })}
      </div>
      <button className="ib-tab-add" type="button" aria-label={t("browser.addTab")} onClick={props.onAdd}>
        <Plus size="var(--icon-md)" aria-hidden="true" />
      </button>
    </div>
  );
}
