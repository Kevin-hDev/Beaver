import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { GpuStatusBadge } from "@/components/agent-local/gpu-status-badge";
import { NAV_ITEMS, type TabId } from "./nav-items";
import "./list-panel-footer.css";

interface ListPanelFooterProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
}

export function ListPanelFooter({ activeTab, onTabChange }: ListPanelFooterProps) {
  const { t } = useTranslation();

  return (
    <div className="lpf">
      {/* La zone garde son nom : la mise au point clavier et les styles de focus
          visent [data-nav-zone="sidebar"], que la navigation soit en colonne ou en rangée. */}
      <nav className="lpf-nav" data-nav-zone="sidebar">
        {NAV_ITEMS.map((item) => {
          const active = activeTab === item.id;
          const label = t(item.i18nKey);
          return (
            <Tooltip key={item.id} label={label} placement="top">
              <button
                type="button"
                className="icon-btn lpf-btn"
                aria-label={label}
                aria-current={active ? "page" : undefined}
                data-nav-active={active ? "true" : undefined}
                onClick={() => onTabChange(item.id)}
              >
                <item.icon />
              </button>
            </Tooltip>
          );
        })}
      </nav>
      <GpuStatusBadge />
    </div>
  );
}
