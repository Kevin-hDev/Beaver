import { useState, useEffect } from "react";
import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";
import { UserCircle, ChatsCircle, Gear } from "@/components/ui/icons";
import type { Icon } from "@phosphor-icons/react";
import { HeartbeatIcon } from "@/components/ui/heartbeat-icon";
import { DragRegion } from "./drag-region";

function useSidebarExpand() {
  const [expand, setExpand] = useState(() => {
    const saved = localStorage.getItem("clgo-sidebar-expand");
    return saved === null ? true : saved === "true";
  });
  useEffect(() => {
    const handler = (e: Event) => setExpand((e as CustomEvent<boolean>).detail);
    window.addEventListener("clgo-sidebar-expand", handler);
    return () => window.removeEventListener("clgo-sidebar-expand", handler);
  }, []);
  return expand;
}

export type TabId = "heartbeat" | "personality" | "agent-local" | "settings";

interface NavItem {
  id: TabId;
  icon?: Icon;
  customIcon?: typeof HeartbeatIcon;
  i18nKey: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "agent-local", icon: ChatsCircle, i18nKey: "nav.agentLocal" },
  { id: "heartbeat", customIcon: HeartbeatIcon, i18nKey: "nav.heartbeat" },
  { id: "personality", icon: UserCircle, i18nKey: "nav.personality" },
];

const ICON_SIZE = "var(--icon-md)";

interface SidebarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
}

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  const { t } = useTranslation();
  const expandOnHover = useSidebarExpand();
  return (
    <nav
      className={cn(
        "group/sb flex flex-col overflow-hidden relative",
        "z-10",
        !expandOnHover && "sb-locked",
      )}
      data-nav-zone="sidebar"
      tabIndex={-1}
      style={{
        width: "var(--sidebar-collapsed)",
        minWidth: "var(--sidebar-collapsed)",
        transition: expandOnHover ? "width 200ms ease-out, min-width 200ms ease-out" : "none",
      }}
      onMouseEnter={expandOnHover ? (e) => {
        e.currentTarget.style.width = "var(--sidebar-expanded)";
        e.currentTarget.style.minWidth = "var(--sidebar-expanded)";
      } : undefined}
      onMouseLeave={expandOnHover ? (e) => {
        e.currentTarget.style.width = "var(--sidebar-collapsed)";
        e.currentTarget.style.minWidth = "var(--sidebar-collapsed)";
      } : undefined}
    >
      <DragRegion />

      {/* Nav items — paddingTop aligne avec "Nouvelle session" du panneau liste */}
      <div className="flex flex-col flex-1" style={{ gap: 2, paddingTop: 8 }}>
        {NAV_ITEMS.map((item) => (
          <div
            key={item.id}
            role="button"
            tabIndex={activeTab === item.id ? 0 : -1}
            aria-current={activeTab === item.id ? "page" : undefined}
            data-nav-active={activeTab === item.id ? "true" : undefined}
            onClick={() => onTabChange(item.id)}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onTabChange(item.id); } }}
            className={cn(
              "relative flex items-center cursor-pointer",
              "whitespace-nowrap",
              "sb-nav-item",
              "transition-all duration-200 ease-out",
            )}
            style={{
              paddingTop: "0.28rem",
              paddingBottom: "0.28rem",
              marginLeft: "0.5rem",
              marginRight: "0.5rem",
              borderRadius: "var(--nav-item-radius)",
              background: activeTab === item.id ? "var(--surface-hover)" : undefined,
            }}
          >
            {item.icon ? (
              <item.icon
                size={ICON_SIZE}
                weight={activeTab === item.id ? "fill" : "regular"}
                className={cn(
                  "shrink-0 text-[var(--ink-muted)]",
                  activeTab === item.id && "text-[var(--ink)]",
                )}
              />
            ) : item.customIcon ? (
              <item.customIcon
                size={ICON_SIZE}
                className={cn(
                  "shrink-0 text-[var(--ink-muted)]",
                  activeTab === item.id && "text-[var(--ink)]",
                )}
              />
            ) : null}
            <span
              className={cn(
                "sb-nav-label text-sm text-[var(--ink-muted)]",
                "w-0 overflow-hidden opacity-0",
                "group-hover/sb:opacity-100",
                "transition-opacity duration-150",
                activeTab === item.id && "text-[var(--ink)] font-medium",
              )}
            >
              {t(item.i18nKey)}
            </span>
          </div>
        ))}
      </div>

      {/* Settings */}
      <div
        role="button"
        tabIndex={activeTab === "settings" ? 0 : -1}
        aria-current={activeTab === "settings" ? "page" : undefined}
        data-nav-active={activeTab === "settings" ? "true" : undefined}
        onClick={() => onTabChange("settings")}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onTabChange("settings"); } }}
        className={cn(
          "flex items-center cursor-pointer",
          "whitespace-nowrap",
          "sb-nav-item",
          "transition-all duration-200 ease-out",
        )}
        style={{
          paddingTop: "0.28rem",
          paddingBottom: "0.28rem",
          marginLeft: "0.5rem",
          marginRight: "0.5rem",
          marginBottom: "0.67rem",
          borderRadius: "var(--nav-item-radius)",
          background: activeTab === "settings" ? "var(--surface-hover)" : undefined,
        }}
      >
        <Gear
          size={ICON_SIZE}
          weight={activeTab === "settings" ? "fill" : "regular"}
          className={cn(
            "shrink-0 text-[var(--ink-muted)]",
            activeTab === "settings" && "text-[var(--ink)]",
          )}
        />
        <span className={cn(
          "sb-nav-label text-sm text-[var(--ink-muted)]",
          "w-0 overflow-hidden opacity-0",
          "group-hover/sb:opacity-100",
          "transition-opacity duration-150",
          activeTab === "settings" && "text-[var(--ink)] font-medium",
        )}>
          {t("nav.settings")}
        </span>
      </div>
    </nav>
  );
}
