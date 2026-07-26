import { useState, useCallback, useRef, type CSSProperties, type ReactNode } from "react";
import { Sidebar, type TabId } from "./sidebar";
import { DragRegion } from "./drag-region";
import { WindowToolbar } from "./window-toolbar";
import { useUpdateChecker } from "@/hooks/use-update-checker";
import { CHAT_MIN_WIDTH } from "@/hooks/file-preview-storage";
import { CHAT_COMPACT_MIN_WIDTH } from "@/hooks/agent-panel-layout-solver";
import { IS_MAC } from "@/lib/platform";
import { GpuStatusBadge } from "@/components/agent-local/gpu-status-badge";
import { WindowControls } from "./window-controls";
import { PanelSlotProvider, PanelSlotTarget } from "./panel-slots";
import { useAgentPanelsAutoSidebar } from "./agent-panels-auto-sidebar";
import { useAppLayoutShortcuts, useSidebarHiddenOffset, useWindowFullscreen } from "./use-app-layout-effects";
import { ModelDownloadBadge } from "./model-download-badge";
import { getListMinWidthPx, nextListPanelWidth } from "./list-panel-width";
import { AppLayoutOverlays } from "./app-layout-overlays";
import {
  INITIAL_AGENT_SIDEBAR_LAYOUT_STATE,
  autoHideAgentSidebar,
  setAgentPanelsTight,
  shouldCompactAgentChat,
  toggleAgentSidebar,
} from "./sidebar-compact-state";
import "./app-layout.css";

const GPU_BADGE_OFFSET = 12;
interface AppLayoutProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  onShowWelcome?: () => void;
  onBack: () => void;
  onForward: () => void;
  canGoBack: boolean;
  canGoForward: boolean;
  onSearchSelect: (sessionId: string) => void;
  onNewSession?: () => void;
  children: ReactNode;
}

export function AppLayout({
  activeTab, onTabChange,
  children,
  onShowWelcome,
  onBack, onForward, canGoBack, canGoForward,
  onSearchSelect, onNewSession,
}: AppLayoutProps) {
  const [agentSidebar, setAgentSidebar] = useState(INITIAL_AGENT_SIDEBAR_LAYOUT_STATE);
  const [searchOpen, setSearchOpen] = useState(false);
  const [updatesOpen, setUpdatesOpen] = useState(false);
  const fullscreen = useWindowFullscreen();
  const updates = useUpdateChecker();
  const sidebarHiddenOffset = useSidebarHiddenOffset(agentSidebar.sidebarOpen);

  const [listWidth, setListWidth] = useState<number | null>(null);
  const dragging = useRef(false);

  const handleResizeStart = useCallback((e: React.PointerEvent) => {
    e.preventDefault();
    dragging.current = true;
    const startX = e.clientX;
    const listEl = (e.target as HTMLElement).closest(".app-sidebar-block")?.querySelector(".app-list-panel") as HTMLElement;
    if (!listEl) return;
    const startListW = listEl.getBoundingClientRect().width;
    const minListW = getListMinWidthPx(listEl, startListW);

    const onMove = (ev: PointerEvent) => {
      const delta = ev.clientX - startX;
      setListWidth(nextListPanelWidth(startListW, minListW, delta));
    };
    const onUp = () => {
      dragging.current = false;
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }, []);

  const openSearch = useCallback(() => setSearchOpen(true), []);
  const closeSearch = useCallback(() => setSearchOpen(false), []);
  const toggleSearch = useCallback(() => setSearchOpen((o) => !o), []);
  const toggleSidebar = useCallback(() => setAgentSidebar(toggleAgentSidebar), []);
  const toggleUpdates = useCallback(() => setUpdatesOpen((o) => !o), []);
  const closeUpdates = useCallback(() => setUpdatesOpen(false), []);
  const handleAutoSidebarHide = useCallback(() => setAgentSidebar(autoHideAgentSidebar), []);
  const handleAgentPanelsTightChange = useCallback((tight: boolean) => {
    setAgentSidebar((state) => setAgentPanelsTight(state, tight));
  }, []);

  useAppLayoutShortcuts({ onBack, onForward, onNewSession, toggleSearch, toggleSidebar });
  useAgentPanelsAutoSidebar(
    agentSidebar.sidebarOpen,
    agentSidebar.manualReveal,
    handleAutoSidebarHide,
    handleAgentPanelsTightChange,
  );
  const compactAgentChat = shouldCompactAgentChat(agentSidebar);
  const chatTargetMinWidth = compactAgentChat ? CHAT_COMPACT_MIN_WIDTH : CHAT_MIN_WIDTH;
  const layoutStyle = {
    "--app-sidebar-hidden-offset": `${sidebarHiddenOffset}px`,
    "--agent-chat-target-min-width": `${chatTargetMinWidth}px`,
    "--agent-chat-min-width": `${chatTargetMinWidth}px`,
  } as CSSProperties;

  return (
    <PanelSlotProvider>
      <div
        className={`app-root ${IS_MAC ? "os-mac" : "os-other"} ${agentSidebar.sidebarOpen ? "" : "sidebar-hidden"} ${compactAgentChat ? "agent-chat-compact" : ""} ${fullscreen ? "is-fullscreen" : ""}`}
        style={layoutStyle}
      >
        <WindowControls />
        <WindowToolbar
          sidebarOpen={agentSidebar.sidebarOpen}
          onToggleSidebar={toggleSidebar}
          onBack={onBack}
          onForward={onForward}
          onNewSession={() => onShowWelcome?.()}
          onSearch={openSearch}
          onToggleUpdates={toggleUpdates}
          updatesCount={updates.totalCount}
          canGoBack={canGoBack}
          canGoForward={canGoForward}
        />
        <div className={`app-sidebar-block ${agentSidebar.sidebarOpen ? "" : "app-sidebar-hidden"}`}>
          <Sidebar activeTab={activeTab} onTabChange={onTabChange} />
          <div
            className="app-list-panel" data-nav-zone="list" tabIndex={-1}
            style={{
              ...(listWidth ? { width: listWidth, minWidth: listWidth } : {}),
              position: "relative",
            }}
          >
            <DragRegion />
            <PanelSlotTarget name="list" />
            <div style={{
              position: "absolute",
              bottom: GPU_BADGE_OFFSET,
              right: GPU_BADGE_OFFSET,
            }}>
              <GpuStatusBadge />
            </div>
          </div>
          <div className="sidebar-resize-handle" onPointerDown={handleResizeStart} />
        </div>
        <div className="app-detail-panel" data-nav-zone="detail" tabIndex={-1}>
          <DragRegion
            height={22}
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              right: 0,
              zIndex: 1,
            }}
          />
          <PanelSlotTarget name="detail" />
          <ModelDownloadBadge />
        </div>
        <AppLayoutOverlays
          searchOpen={searchOpen}
          updatesOpen={updatesOpen}
          onCloseSearch={closeSearch}
          onCloseUpdates={closeUpdates}
          onSearchSelect={onSearchSelect}
          updates={updates}
        />
        {children}
      </div>
    </PanelSlotProvider>
  );
}
