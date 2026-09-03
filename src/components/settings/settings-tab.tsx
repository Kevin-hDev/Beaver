"use no memo";
import { useCallback, useState, useMemo, memo } from "react";
import { useSettings } from "@/hooks/use-settings";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import type { ThemeChoice } from "@/hooks/use-theme";
import { GeneralSettings } from "./general-settings";
import { AdvancedSettings } from "./advanced-settings";
import { ToolsSettings } from "./tools-settings";
import { MemorySettings } from "./memory-settings";
import { SystemPromptSettings } from "./system-prompt-settings";
import { ForecastSettings } from "./forecast-settings";
import { ArchivedChatsSettings } from "./archived-chats-settings";
import { ShortcutsSettings } from "./shortcuts-settings";
import { AboutSettings } from "./about-settings";
import { UpdatesSettings } from "./updates-settings";
import { MascotSettings } from "./mascot-settings";
import { LlmExplorer } from "./llm-explorer";
import { useResolvedSettingsSections } from "./settings-sections";
import { SettingsSubTabList } from "./settings-subtab-list";
import { PanelSlot } from "@/components/layout/panel-slots";
import { useSlotOccupantByTarget } from "@/features/extension-ui/slot-contexts";
import { SlotRenderer } from "@/features/extension-ui/slot-renderer";
import type { DeepPartial, SettingsNavState, SettingsSubTab } from "@/types/navigation";
import {
  SettingsChildSlots,
  usesSettingsChildSlots,
} from "./settings-child-slots";
import "./settings-tab.css";

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
  const [childDetailTarget, setChildDetailTarget] = useState<HTMLElement | null>(null);

  const setSubTab = useCallback((id: SettingsSubTab) => {
    onNavChange({ subTab: id });
  }, [onNavChange]);
  const handleAdvancedFocusTarget = useCallback(() => {
    onNavReplace({ advancedTarget: null });
  }, [onNavReplace]);
  const subTab = navState.subTab;
  const selectedOccupant = useSlotOccupantByTarget(subTab, "settingsTab");
  const sections = useResolvedSettingsSections();
  const tabIds = useMemo(
    () => sections.flatMap((section) => section.tabs).map((tab) => tab.id),
    [sections],
  );
  useArrowNavigation({
    items: tabIds,
    selectedId: subTab,
    onSelect: setSubTab,
    enabled: listFocused,
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const settings = useSettings();

  const list = useMemo(
    () => <SettingsSubTabList active={subTab} onSelect={setSubTab} />,
    [setSubTab, subTab],
  );

  const detailContent = useMemo(() => {
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
    if (subTab === "system-prompt") return <SystemPromptSettings />;
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
    if (subTab === "advanced") {
      return (
        <AdvancedSettings
          focusTarget={navState.advancedTarget}
          onFocusTargetHandled={handleAdvancedFocusTarget}
        />
      );
    }
    if (subTab === "shortcuts") return <ShortcutsSettings />;
    if (subTab === "updates") return <UpdatesSettings />;
    if (subTab === "about") return <AboutSettings />;
    return null;
  }, [activeSessionId, handleAdvancedFocusTarget, navState, onNavChange, onNavReplace, onThemeChange, settings, subTab, themeChoice]);
  const detail = selectedOccupant ? (
    <SlotRenderer
      placement={selectedOccupant.placement}
      occupantId={selectedOccupant.id}
      context={detailContent}
      render={(_occupant, content) => content}
    />
  ) : null;

  return (
    <>
      <PanelSlot name="list">{list}</PanelSlot>
      <PanelSlot name="detail">{detail}</PanelSlot>
      <SettingsChildSlots
        subTab={subTab}
        navState={navState}
        onNavChange={onNavChange}
        onNavReplace={onNavReplace}
        target={childDetailTarget}
      />
    </>
  );
});
