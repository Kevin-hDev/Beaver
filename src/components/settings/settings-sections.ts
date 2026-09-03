import { useMemo, type ComponentType } from "react";
import {
  AboutIcon, AdvancedIcon, ArchivedChatsIcon, ChannelsIcon, ConnectorsIcon,
  ExtensionsIcon, GeneralIcon, LlmIcon, MascotIcon, MemoryIcon,
  OllamaIcon, ProvidersIcon, ShortcutsIcon, SystemPromptIcon, ToolsIcon, UpdatesIcon,
} from "./settings-tab-icons";
import type { SettingsTabIconProps } from "./settings-tab-icons";
import { ForecastIcon } from "@/components/ui/forecast-icon";
import { coreOccupantsFor } from "@/features/extension-ui/core-occupants";
import { useSlotOccupants } from "@/features/extension-ui/slot-contexts";
import { SETTINGS_NAVIGATION_PLACEMENTS } from "@/features/extension-ui/slot-navigation";
import type { CoreSettingsTabId, SlotOccupant } from "@/features/extension-ui/slot-types";
import type { SettingsSubTab } from "@/types/navigation";

export interface SubTabDef {
  id: SettingsSubTab;
  i18n: string;
  icon: ComponentType<SettingsTabIconProps>;
  occupantId: string;
}

export interface SettingsSection {
  i18n: string;
  tabs: SubTabDef[];
}

export const SETTINGS_SECTIONS: readonly SettingsSection[] = projectSettingsSections(
  SETTINGS_NAVIGATION_PLACEMENTS.map((placement) => coreOccupantsFor(placement)),
);
export const SETTINGS_TAB_IDS: readonly SettingsSubTab[] = SETTINGS_SECTIONS
  .flatMap((section) => section.tabs)
  .map((tab) => tab.id);

export function useResolvedSettingsSections(): readonly SettingsSection[] {
  const preferences = useSlotOccupants("settings.navigation.preferences");
  const agent = useSlotOccupants("settings.navigation.agent");
  const models = useSlotOccupants("settings.navigation.models");
  const integrations = useSlotOccupants("settings.navigation.integrations");
  const application = useSlotOccupants("settings.navigation.application");
  return useMemo(
    () => projectSettingsSections([preferences, agent, models, integrations, application]),
    [agent, application, integrations, models, preferences],
  );
}

function projectSettingsSections(
  groups: readonly (readonly SlotOccupant[])[],
): readonly SettingsSection[] {
  return groups.flatMap((occupants) => {
    if (occupants.length === 0) return [];
    const i18n = occupants[0].sectionLabelKey;
    if (!i18n || occupants.some((occupant) => occupant.sectionLabelKey !== i18n)) {
      throw new Error("Invalid core settings section.");
    }
    return [{ i18n, tabs: occupants.map(subTabFromOccupant) }];
  });
}

function subTabFromOccupant(occupant: SlotOccupant): SubTabDef {
  if (occupant.source.kind !== "core" || !occupant.labelKey || !occupant.iconKey) {
    throw new Error("Invalid core settings occupant.");
  }
  return {
    id: occupant.target as CoreSettingsTabId,
    i18n: occupant.labelKey,
    icon: settingsIcon(occupant.iconKey),
    occupantId: occupant.id,
  };
}

function settingsIcon(iconKey: string): ComponentType<SettingsTabIconProps> {
  if (iconKey === "general") return GeneralIcon;
  if (iconKey === "mascot") return MascotIcon;
  if (iconKey === "shortcuts") return ShortcutsIcon;
  if (iconKey === "memory") return MemoryIcon;
  if (iconKey === "system-prompt") return SystemPromptIcon;
  if (iconKey === "tools") return ToolsIcon;
  if (iconKey === "advanced") return AdvancedIcon;
  if (iconKey === "ollama") return OllamaIcon;
  if (iconKey === "forecast") return ForecastIcon;
  if (iconKey === "llm") return LlmIcon;
  if (iconKey === "providers") return ProvidersIcon;
  if (iconKey === "connectors") return ConnectorsIcon;
  if (iconKey === "channels") return ChannelsIcon;
  if (iconKey === "extensions") return ExtensionsIcon;
  if (iconKey === "updates") return UpdatesIcon;
  if (iconKey === "archived-chats") return ArchivedChatsIcon;
  if (iconKey === "about") return AboutIcon;
  throw new Error("Unknown core settings icon.");
}
