import { UserCircle, ChatsCircle, Gear } from "@/components/ui/icons";
import type { Icon } from "@phosphor-icons/react";
import { HeartbeatIcon } from "@/components/ui/heartbeat-icon";

export type TabId = "heartbeat" | "personality" | "agent-local" | "settings";

export interface NavItem {
  id: TabId;
  icon?: Icon;
  customIcon?: typeof HeartbeatIcon;
  i18nKey: string;
}

/* Les Réglages ferment la rangée au même titre que les sections. Ils vivaient à
   part dans le rail, qui les collait en bas d'une colonne verticale. */
export const NAV_ITEMS: NavItem[] = [
  { id: "agent-local", icon: ChatsCircle, i18nKey: "nav.agentLocal" },
  { id: "heartbeat", customIcon: HeartbeatIcon, i18nKey: "nav.heartbeat" },
  { id: "personality", icon: UserCircle, i18nKey: "nav.personality" },
  { id: "settings", icon: Gear, i18nKey: "nav.settings" },
];
