import { useTranslation } from "react-i18next";
import { SidebarToggleIcon, ArrowLeftIcon, ArrowRightIcon, SearchIcon } from "./toolbar-icons";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { Tooltip } from "@/components/ui/tooltip";
import { IS_MAC, MOD, ALT } from "@/lib/platform";
import updateIcon from "@/assets/update.png";
import { SlotRenderer } from "@/features/extension-ui/slot-renderer";
import type { SlotOccupant } from "@/features/extension-ui/slot-types";
import {
  StandardPlacementAction,
  useStandardEntry,
} from "@/features/extension-ui/standard/standard-contributions";
import { AdvancedMountAnchor } from "@/features/extension-ui/advanced/advanced-mount-anchor";
import "./window-toolbar.css";

interface WindowToolbarProps {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onBack: () => void;
  onForward: () => void;
  onNewSession: () => void;
  onSearch: () => void;
  onToggleUpdates: () => void;
  updatesCount: number;
  canGoBack: boolean;
  canGoForward: boolean;
}

export function WindowToolbar({
  sidebarOpen, onToggleSidebar,
  onBack, onForward, onNewSession, onSearch,
  onToggleUpdates, updatesCount,
  canGoBack, canGoForward,
}: WindowToolbarProps) {
  const { t } = useTranslation();
  const context: ToolbarRenderContext = {
    sidebarOpen, onToggleSidebar, onBack, onForward, onNewSession, onSearch,
    onToggleUpdates, updatesCount, canGoBack, canGoForward,
    translate: (key) => t(key),
  };

  return (
    <div className={`window-toolbar${IS_MAC ? " is-mac" : ""}`}>
      <SlotRenderer
        placement="app.toolbar.primary"
        context={context}
        render={(occupant, current) => (
          <ToolbarOccupant occupant={occupant} context={current} />
        )}
      />
      <AdvancedMountAnchor placement="app.toolbar.primary" />
    </div>
  );
}

function ToolbarOccupant({
  occupant,
  context,
}: {
  occupant: SlotOccupant;
  context: ToolbarRenderContext;
}) {
  const entry = useStandardEntry(occupant);
  return entry
    ? <StandardPlacementAction entry={entry} surface="toolbar" />
    : renderToolbarOccupant(occupant, context);
}

interface ToolbarRenderContext extends WindowToolbarProps {
  translate: (key: string) => string;
}

function renderToolbarOccupant(occupant: SlotOccupant, context: ToolbarRenderContext) {
  const t = context.translate;
  if (occupant.target === "toggle-sidebar") {
    return (
      <Tooltip label={`${t("settings.shortcuts.toggleSidebar")} (${MOD}B)`}>
        <button className="icon-btn toolbar-btn" onClick={context.onToggleSidebar}>
          <SidebarToggleIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
    );
  }
  if (occupant.target === "back") {
    return (
      <Tooltip label={`${t("settings.shortcuts.goBack")} (${MOD}◀)`}>
        <button className="icon-btn toolbar-btn" onClick={context.onBack} disabled={!context.canGoBack}>
          <ArrowLeftIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
    );
  }
  if (occupant.target === "forward") {
    return (
      <Tooltip label={`${t("settings.shortcuts.goForward")} (${MOD}▶)`}>
        <button className="icon-btn toolbar-btn" onClick={context.onForward} disabled={!context.canGoForward}>
          <ArrowRightIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
    );
  }
  if (occupant.target === "search") {
    return (
      <Tooltip label={`${t("settings.shortcuts.searchDialog")} (${MOD}G)`}>
        <button className="icon-btn toolbar-btn" onClick={context.onSearch}>
          <SearchIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
    );
  }
  if (occupant.target === "updates" && context.sidebarOpen && context.updatesCount > 0) {
    return (
        <Tooltip label={t("updates.tooltip")}>
          <button className="icon-btn toolbar-btn toolbar-btn-update" onClick={context.onToggleUpdates}>
            <img src={updateIcon} alt="" style={{ width: "var(--chrome-icon-md)", height: "var(--chrome-icon-md)" }} />
          </button>
        </Tooltip>
    );
  }
  if (occupant.target === "new-session" && !context.sidebarOpen) {
    return (
        <Tooltip label={`${t("settings.shortcuts.newSession")} (${ALT}${MOD}N)`}>
          <button className="icon-btn toolbar-btn" onClick={context.onNewSession}>
            <ComposeIcon size="var(--chrome-icon-md)" />
          </button>
        </Tooltip>
    );
  }
  return null;
}
