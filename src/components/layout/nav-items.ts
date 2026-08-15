import { BeaverIcon, HeartbeatIcon, SettingsIcon } from "./nav-tab-icons";
import { SessionIcon } from "@/components/ui/session-icon";
import type { InlineIconProps } from "@/components/ui/inline-icon";
import type { ComponentType } from "react";

export type TabId = "heartbeat" | "personality" | "agent-local" | "settings";

export interface NavItem {
  id: TabId;
  icon: ComponentType<InlineIconProps>;
  i18nKey: string;
}

/* Les Réglages ferment la rangée au même titre que les sections. Ils vivaient à
   part dans le rail, qui les collait en bas d'une colonne verticale. */
export const NAV_ITEMS: NavItem[] = [
  { id: "agent-local", icon: SessionIcon, i18nKey: "nav.agentLocal" },
  { id: "heartbeat", icon: HeartbeatIcon, i18nKey: "nav.heartbeat" },
  { id: "personality", icon: BeaverIcon, i18nKey: "nav.personality" },
  { id: "settings", icon: SettingsIcon, i18nKey: "nav.settings" },
];
