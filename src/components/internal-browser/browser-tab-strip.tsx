import { useTranslation } from "react-i18next";
import { Plus, X } from "@/components/ui/icons";
import { BrowserIcon } from "./browser-icon";
import type { BrowserTabState } from "./browser-types";

interface BrowserTabStripProps {
  tabs: BrowserTabState[];
  activeTabId: string;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
  onAdd: () => void;
}

export function BrowserTabStrip(props: BrowserTabStripProps) {
  const { t } = useTranslation();
  return (
    <div className="ib-tabs-bar">
      <div className="ib-tabs-scroll" role="tablist" aria-label={t("browser.tabsLabel")}>
        {props.tabs.map((tab) => {
          const active = tab.id === props.activeTabId;
          const title = tab.title || t("browser.newTab");
          return (
            <div className={`ib-tab ${active ? "ib-tab-active" : ""}`} key={tab.id}>
              <button
                className="ib-tab-select"
                type="button"
                role="tab"
                aria-selected={active}
                title={title}
                onClick={() => props.onSelect(tab.id)}
              >
                <BrowserIcon className="ib-tab-icon" />
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
