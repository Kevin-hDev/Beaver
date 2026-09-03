import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { GpuStatusBadge } from "@/components/agent-local/gpu-status-badge";
import { navItemFromOccupant, type TabId } from "./nav-items";
import { SlotRenderer } from "@/features/extension-ui/slot-renderer";
import {
  StandardNavigationButton,
  useStandardEntry,
} from "@/features/extension-ui/standard/standard-contributions";
import type { SlotOccupant } from "@/features/extension-ui/slot-types";
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
          render={(occupant, current) => (
            <NavigationOccupant occupant={occupant} context={current} />
          )}
        />
      </nav>
      <GpuStatusBadge />
    </div>
  );
}

function NavigationOccupant({
  occupant,
  context,
}: {
  occupant: SlotOccupant;
  context: { activeTab: TabId; onTabChange: (tab: TabId) => void; t: (key: string) => string };
}) {
  const entry = useStandardEntry(occupant);
  if (entry) {
    return (
      <StandardNavigationButton
        entry={entry}
        active={context.activeTab === occupant.id}
        onSelect={() => context.onTabChange(occupant.id as TabId)}
      />
    );
  }
  const item = navItemFromOccupant(occupant);
  const active = context.activeTab === item.id;
  const label = context.t(item.i18nKey);
  return (
    <Tooltip label={label} placement="top">
      <button
        type="button"
        className="icon-btn lpf-btn"
        aria-label={label}
        aria-current={active ? "page" : undefined}
        data-nav-active={active ? "true" : undefined}
        onClick={() => context.onTabChange(item.id)}
      >
        <item.icon />
      </button>
    </Tooltip>
  );
}
