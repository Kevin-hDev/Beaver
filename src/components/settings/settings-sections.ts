import {
  AboutIcon, AdvancedIcon, ArchivedChatsIcon, ChannelsIcon, ConnectorsIcon,
  ExtensionsIcon, ForecastIcon, GeneralIcon, LlmIcon, MascotIcon, MemoryIcon,
  OllamaIcon, ProvidersIcon, ShortcutsIcon, SystemPromptIcon, ToolsIcon,
} from "./settings-tab-icons";
import type { SettingsTabIconProps } from "./settings-tab-icons";
import type { ComponentType } from "react";
import type { SettingsSubTab } from "@/types/navigation";

export interface SubTabDef {
  id: SettingsSubTab;
  i18n: string;
  icon: ComponentType<SettingsTabIconProps>;
}

interface SettingsSection {
  i18n: string;
  tabs: SubTabDef[];
}

/* Les onglets sont rangés selon ce qu'on configure à l'intérieur, jamais selon
   l'usage qu'on en fait : Ollama et Forecast installent et paramètrent tous
   deux des modèles locaux, même si l'un produit du texte et l'autre des
   prévisions, donc ils voisinent. Providers rejoint les intégrations parce
   qu'on y saisit une clé et une connexion, pas un modèle. */
export const SETTINGS_SECTIONS: readonly SettingsSection[] = [
  {
    i18n: "settings.sections.preferences",
    tabs: [
      { id: "general", i18n: "settings.tabs.general", icon: GeneralIcon },
      { id: "mascot", i18n: "settings.tabs.mascot", icon: MascotIcon },
      { id: "shortcuts", i18n: "settings.tabs.shortcuts", icon: ShortcutsIcon },
    ],
  },
  {
    i18n: "settings.sections.agent",
    tabs: [
      { id: "memory", i18n: "settings.tabs.memory", icon: MemoryIcon },
      { id: "system-prompt", i18n: "settings.tabs.systemPrompt", icon: SystemPromptIcon },
      { id: "tools", i18n: "settings.tabs.tools", icon: ToolsIcon },
      { id: "advanced", i18n: "settings.tabs.advanced", icon: AdvancedIcon },
    ],
  },
  {
    i18n: "settings.sections.models",
    tabs: [
      { id: "ollama", i18n: "settings.tabs.ollama", icon: OllamaIcon },
      { id: "forecast", i18n: "forecast.title", icon: ForecastIcon },
      { id: "llm", i18n: "settings.tabs.llm", icon: LlmIcon },
    ],
  },
  {
    i18n: "settings.sections.integrations",
    tabs: [
      { id: "providers", i18n: "settings.tabs.providers", icon: ProvidersIcon },
      { id: "connectors", i18n: "settings.tabs.connectors", icon: ConnectorsIcon },
      { id: "channels", i18n: "settings.tabs.channels", icon: ChannelsIcon },
      { id: "extensions", i18n: "settings.tabs.extensions", icon: ExtensionsIcon },
    ],
  },
  {
    i18n: "settings.sections.application",
    tabs: [
      { id: "archived-chats", i18n: "settings.tabs.archivedChats", icon: ArchivedChatsIcon },
      { id: "about", i18n: "settings.tabs.about", icon: AboutIcon },
    ],
  },
];

/* Ordre de parcours aux flèches : celui de l'affichage, sections aplaties.
   Les en-têtes n'y figurent pas — ils ne sont pas des destinations. */
export const SETTINGS_TAB_IDS: readonly SettingsSubTab[] = SETTINGS_SECTIONS
  .flatMap((section) => section.tabs)
  .map((tab) => tab.id);
