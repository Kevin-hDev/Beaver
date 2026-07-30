import {
  Archive, Brain, GearSix, Key, Sliders, Info, BookOpenText, Keyboard,
  Plugs, Broadcast, ChartLineUp, Wrench, PawPrint, PuzzlePiece,
} from "@/components/ui/icons";
import ollamaDark from "@/assets/ollama.png";
import ollamaLight from "@/assets/ollama-light.png";
import type { Icon } from "@phosphor-icons/react";
import type { SettingsSubTab } from "@/types/navigation";

export interface SubTabDef {
  id: SettingsSubTab;
  i18n: string;
  icon?: Icon;
  imgDark?: string;
  imgLight?: string;
}

export interface SettingsSection {
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
      { id: "general", i18n: "settings.tabs.general", icon: GearSix },
      { id: "mascot", i18n: "settings.tabs.mascot", icon: PawPrint },
      { id: "shortcuts", i18n: "settings.tabs.shortcuts", icon: Keyboard },
    ],
  },
  {
    i18n: "settings.sections.agent",
    tabs: [
      { id: "memory", i18n: "settings.tabs.memory", icon: Brain },
      { id: "tools", i18n: "settings.tabs.tools", icon: Wrench },
      { id: "advanced", i18n: "settings.tabs.advanced", icon: Sliders },
    ],
  },
  {
    i18n: "settings.sections.models",
    tabs: [
      { id: "ollama", i18n: "settings.tabs.ollama", imgDark: ollamaDark, imgLight: ollamaLight },
      { id: "forecast", i18n: "forecast.title", icon: ChartLineUp },
      { id: "llm", i18n: "settings.tabs.llm", icon: BookOpenText },
    ],
  },
  {
    i18n: "settings.sections.integrations",
    tabs: [
      { id: "providers", i18n: "settings.tabs.providers", icon: Key },
      { id: "connectors", i18n: "settings.tabs.connectors", icon: Plugs },
      { id: "channels", i18n: "settings.tabs.channels", icon: Broadcast },
      { id: "extensions", i18n: "settings.tabs.extensions", icon: PuzzlePiece },
    ],
  },
  {
    i18n: "settings.sections.application",
    tabs: [
      { id: "archived-chats", i18n: "settings.tabs.archivedChats", icon: Archive },
      { id: "about", i18n: "settings.tabs.about", icon: Info },
    ],
  },
];

/* Ordre de parcours aux flèches : celui de l'affichage, sections aplaties.
   Les en-têtes n'y figurent pas — ils ne sont pas des destinations. */
export const SETTINGS_TAB_IDS: readonly SettingsSubTab[] = SETTINGS_SECTIONS
  .flatMap((section) => section.tabs)
  .map((tab) => tab.id);
