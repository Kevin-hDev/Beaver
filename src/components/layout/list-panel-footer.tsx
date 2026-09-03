import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { GpuStatusBadge } from "@/components/agent-local/gpu-status-badge";
import { navItemFromOccupant, type TabId } from "./nav-items";
import { SlotRenderer } from "@/features/extension-ui/slot-renderer";
import "./list-panel-footer.css";

interface ListPanelFooterProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
}

export function ListPanelFooter({ activeTab, onTabChange }: ListPanelFooterProps) {
  const { t } = useTranslation();
  const context = { activeTab, onTabChange, t };

  return (
    <div className="lpf">
      {/* La zone garde son nom : la mise au point clavier et les styles de focus
          visent [data-nav-zone="sidebar"], que la navigation soit en colonne ou en rangée. */}
      <nav className="lpf-nav" data-nav-zone="sidebar">
        <SlotRenderer
          placement="app.navigation.primary"
          context={context}
          render={(occupant, current) => {
            const item = navItemFromOccupant(occupant);
            const active = current.activeTab === item.id;
            const label = current.t(item.i18nKey);
            return (
              <Tooltip key={item.id} label={label} placement="top">
                <button
                  type="button"
                  className="icon-btn lpf-btn"
                  aria-label={label}
                  aria-current={active ? "page" : undefined}
                  data-nav-active={active ? "true" : undefined}
                  onClick={() => current.onTabChange(item.id)}
                >
                  <item.icon />
                </button>
              </Tooltip>
            );
          }}
        />
      </nav>
      <GpuStatusBadge />
    </div>
  );
}
