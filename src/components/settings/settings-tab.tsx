"use no memo";
import { useCallback, useState, useMemo, memo } from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "@/hooks/use-settings";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import type { ThemeChoice } from "@/hooks/use-theme";
import { Archive, Brain, GearSix, Key, Sliders, Info, BookOpenText, Keyboard, Plugs, Broadcast, ChartLineUp, Wrench, PawPrint, PuzzlePiece } from "@/components/ui/icons";
import { ThemedIcon } from "@/components/ui/themed-icon";
import { GeneralSettings } from "./general-settings";
import { AdvancedSettings } from "./advanced-settings";
import { ToolsSettings } from "./tools-settings";
import { MemorySettings } from "./memory-settings";
import { ForecastSettings } from "./forecast-settings";
import { ArchivedChatsSettings } from "./archived-chats-settings";
import { ShortcutsSettings } from "./shortcuts-settings";
import { AboutSettings } from "./about-settings";
import { MascotSettings } from "./mascot-settings";
import { LlmExplorer } from "./llm-explorer";
import ollamaDark from "@/assets/ollama.png";
import ollamaLight from "@/assets/ollama-light.png";
import type { Icon } from "@phosphor-icons/react";
import { PanelSlot } from "@/components/layout/panel-slots";
import type { DeepPartial, SettingsNavState, SettingsSubTab } from "@/types/navigation";
import {
  SettingsChildSlots,
  usesSettingsChildSlots,
} from "./settings-child-slots";
import "./settings-tab.css";

interface SubTabDef {
  id: SettingsSubTab;
  i18n: string;
  icon?: Icon;
  imgDark?: string;
  imgLight?: string;
}

const SUB_TABS: SubTabDef[] = [
  { id: "general", i18n: "settings.tabs.general", icon: GearSix },
  { id: "advanced", i18n: "settings.tabs.advanced", icon: Sliders },
  { id: "memory", i18n: "settings.tabs.memory", icon: Brain },
  { id: "tools", i18n: "settings.tabs.tools", icon: Wrench },
  { id: "mascot", i18n: "settings.tabs.mascot", icon: PawPrint },
  { id: "ollama", i18n: "settings.tabs.ollama", imgDark: ollamaDark, imgLight: ollamaLight },
  { id: "providers", i18n: "settings.tabs.providers", icon: Key },
  { id: "connectors", i18n: "settings.tabs.connectors", icon: Plugs },
  { id: "extensions", i18n: "settings.tabs.extensions", icon: PuzzlePiece },
  { id: "channels", i18n: "settings.tabs.channels", icon: Broadcast },
  { id: "forecast", i18n: "forecast.title", icon: ChartLineUp },
  { id: "llm", i18n: "settings.tabs.llm", icon: BookOpenText },
  { id: "archived-chats", i18n: "settings.tabs.archivedChats", icon: Archive },
  { id: "shortcuts", i18n: "settings.tabs.shortcuts", icon: Keyboard },
  { id: "about", i18n: "settings.tabs.about", icon: Info },
];

interface SettingsTabProps {
  themeChoice: ThemeChoice;
  onThemeChange: (theme: ThemeChoice) => void;
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
  listFocused?: boolean;
  activeSessionId?: string | null;
}

export const SettingsTab = memo(function SettingsTab({
  themeChoice,
  onThemeChange,
  navState,
  onNavChange,
  onNavReplace,
  listFocused = true,
  activeSessionId,
}: SettingsTabProps) {
  const [childListTarget, setChildListTarget] = useState<HTMLElement | null>(null);
  const [childDetailTarget, setChildDetailTarget] = useState<HTMLElement | null>(null);

  const setSubTab = useCallback((id: SettingsSubTab) => {
    onNavChange({ subTab: id });
  }, [onNavChange]);
  const subTab = navState.subTab;
  const subTabIds = useMemo(() => SUB_TABS.map((t) => t.id), []);
  useArrowNavigation({
    items: subTabIds,
    selectedId: subTab,
    onSelect: setSubTab,
    enabled: listFocused,
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const settings = useSettings();
  const { t } = useTranslation();

  const list = useMemo(() => (
    <div style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0, overflow: "hidden" }}>
      <div
        style={{
          padding: "var(--space-sm) var(--space-sm) var(--space-sm) max(0px, calc(var(--space-sm) - 4px))",
          flexShrink: 0,
        }}
      >
        {SUB_TABS.map((tab) => (
          <div
            key={tab.id}
            role="button"
            tabIndex={subTab === tab.id ? 0 : -1}
            data-nav-active={subTab === tab.id ? "true" : undefined}
            onClick={() => setSubTab(tab.id)}
            onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); setSubTab(tab.id); } }}
            className={`settings-subtab${subTab === tab.id ? " active" : ""}`}
          >
            {tab.icon ? (
              <tab.icon
                size="var(--icon-md)"
                weight={subTab === tab.id ? "fill" : "regular"}
                style={{ flexShrink: 0 }}
              />
            ) : tab.imgDark && tab.imgLight ? (
              <ThemedIcon
                darkSrc={tab.imgDark}
                lightSrc={tab.imgLight}
                size="var(--icon-md)"
                style={{
                  flexShrink: 0,
                  opacity: subTab === tab.id ? 1 : 0.6,
                }}
              />
            ) : null}
            <span className="settings-subtab-label">{t(tab.i18n)}</span>
          </div>
        ))}
      </div>
      <div
        ref={setChildListTarget}
        style={{ display: "flex", flexDirection: "column", flex: 1, minHeight: 0, overflow: "hidden" }}
      >
      </div>
    </div>
  ), [setSubTab, subTab, t]);

  const detail = useMemo(() => {
    if (subTab === "general") {
      return (
        <GeneralSettings
          themeChoice={themeChoice}
          onThemeChange={onThemeChange}
          settings={settings}
        />
      );
    }
    if (usesSettingsChildSlots(subTab)) {
      return (
        <div
          ref={setChildDetailTarget}
          style={{ display: "flex", flex: 1, minHeight: 0, minWidth: 0 }}
        />
      );
    }
    if (subTab === "llm") {
      return <LlmExplorer navState={navState.llmView} onNavChange={(llmView) => onNavChange({ llmView })} />;
    }
    if (subTab === "tools") return <ToolsSettings />;
    if (subTab === "memory") return <MemorySettings activeSessionId={activeSessionId} />;
    if (subTab === "mascot") return <MascotSettings />;
    if (subTab === "forecast") {
      return (
        <ForecastSettings
          navState={navState}
          onNavChange={onNavChange}
          onNavReplace={onNavReplace}
        />
      );
    }
    if (subTab === "archived-chats") return <ArchivedChatsSettings />;
    if (subTab === "advanced") return <AdvancedSettings />;
    if (subTab === "shortcuts") return <ShortcutsSettings />;
    if (subTab === "about") return <AboutSettings />;
    return null;
  }, [activeSessionId, navState, onNavChange, onNavReplace, onThemeChange, settings, subTab, themeChoice]);

  return (
    <>
      <PanelSlot name="list">{list}</PanelSlot>
      <PanelSlot name="detail">{detail}</PanelSlot>
      <SettingsChildSlots
        subTab={subTab}
        navState={navState}
        onNavChange={onNavChange}
        onNavReplace={onNavReplace}
        listTarget={childListTarget}
        detailTarget={childDetailTarget}
      />
    </>
  );
});
