import { BeaverIcon, HeartbeatIcon, SettingsIcon } from "./nav-tab-icons";
import { SessionIcon } from "@/components/ui/session-icon";
import type { InlineIconProps } from "@/components/ui/inline-icon";
import type { ComponentType } from "react";
import { coreOccupantsFor } from "@/features/extension-ui/core-occupants";
import type {
  CoreMainTabId,
  MainTabId,
  SlotOccupant,
} from "@/features/extension-ui/slot-types";

export type TabId = MainTabId;

export interface NavItem {
  id: TabId;
  icon: ComponentType<InlineIconProps>;
  i18nKey: string;
}

export const NAV_ITEMS: readonly NavItem[] = coreOccupantsFor("app.navigation.primary")
  .flatMap((occupant) => {
    const item = navItemFromOccupant(occupant);
    return item ? [item] : [];
  });

export function navItemFromOccupant(occupant: SlotOccupant): NavItem | null {
  if (occupant.source.kind !== "core" || !occupant.labelKey || !occupant.iconKey) {
    return null;
  }
  const icon = navIcon(occupant.iconKey);
  if (!icon) return null;
  return {
    id: occupant.target as CoreMainTabId,
    icon,
    i18nKey: occupant.labelKey,
  };
}

function navIcon(iconKey: string): ComponentType<InlineIconProps> | null {
  if (iconKey === "agent-local") return SessionIcon;
  if (iconKey === "heartbeat") return HeartbeatIcon;
  if (iconKey === "personality") return BeaverIcon;
  if (iconKey === "settings") return SettingsIcon;
  return null;
}
