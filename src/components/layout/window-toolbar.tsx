import { useTranslation } from "react-i18next";
import { SidebarToggleIcon, ArrowLeftIcon, ArrowRightIcon, SearchIcon } from "./toolbar-icons";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { Tooltip } from "@/components/ui/tooltip";
import { IS_MAC, MOD, ALT } from "@/lib/platform";
import updateIcon from "@/assets/update.png";
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

  return (
    <div className={`window-toolbar${IS_MAC ? " is-mac" : ""}`}>
      <Tooltip label={`${t("settings.shortcuts.toggleSidebar")} (${MOD}B)`}>
        <button className="icon-btn toolbar-btn" onClick={onToggleSidebar}>
          <SidebarToggleIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
      <Tooltip label={`${t("settings.shortcuts.goBack")} (${MOD}◀)`}>
        <button className="icon-btn toolbar-btn" onClick={onBack} disabled={!canGoBack}>
          <ArrowLeftIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
      <Tooltip label={`${t("settings.shortcuts.goForward")} (${MOD}▶)`}>
        <button className="icon-btn toolbar-btn" onClick={onForward} disabled={!canGoForward}>
          <ArrowRightIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
      <Tooltip label={`${t("settings.shortcuts.searchDialog")} (${MOD}G)`}>
        <button className="icon-btn toolbar-btn" onClick={onSearch}>
          <SearchIcon size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
      {sidebarOpen && updatesCount > 0 && (
        <Tooltip label={t("updates.tooltip")}>
          <button className="icon-btn toolbar-btn toolbar-btn-update" onClick={onToggleUpdates}>
            <img src={updateIcon} alt="" style={{ width: "var(--chrome-icon-md)", height: "var(--chrome-icon-md)" }} />
          </button>
        </Tooltip>
      )}
      {!sidebarOpen && (
        <Tooltip label={`${t("settings.shortcuts.newSession")} (${ALT}${MOD}N)`}>
          <button className="icon-btn toolbar-btn" onClick={onNewSession}>
            <ComposeIcon size="var(--chrome-icon-md)" />
          </button>
        </Tooltip>
      )}
    </div>
  );
}
