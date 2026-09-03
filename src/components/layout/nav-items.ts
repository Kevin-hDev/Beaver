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
  .map(navItemFromOccupant);

export function navItemFromOccupant(occupant: SlotOccupant): NavItem {
  if (occupant.source.kind !== "core" || !occupant.labelKey || !occupant.iconKey) {
    throw new Error("Invalid core navigation occupant.");
  }
  return {
    id: occupant.target as CoreMainTabId,
    icon: navIcon(occupant.iconKey),
    i18nKey: occupant.labelKey,
  };
}

function navIcon(iconKey: string): ComponentType<InlineIconProps> {
  if (iconKey === "agent-local") return SessionIcon;
  if (iconKey === "heartbeat") return HeartbeatIcon;
  if (iconKey === "personality") return BeaverIcon;
  if (iconKey === "settings") return SettingsIcon;
  throw new Error("Unknown core navigation icon.");
}
