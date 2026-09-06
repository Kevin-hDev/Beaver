import { useCallback, useEffect, useMemo } from "react";
import { AppLayout } from "./app-layout";
import { CoreMainTabContent } from "./core-main-tab-content";
import { VaultErrorBanner } from "./vault-error-banner";
import { useTabHistory } from "@/hooks/use-tab-history";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import { usePanelFocus } from "@/hooks/use-panel-focus";
import { useAgentSessionWorkspace } from "@/hooks/use-agent-session-workspace";
import { useNavigationAvailability } from "@/features/extension-ui/slot-contexts";
import { AppNavigationActionsProvider } from "@/hooks/use-app-navigation-actions";
import type { ThemeChoice } from "@/hooks/use-theme";
import type { TabId } from "./nav-items";
import {
  DEFAULT_APP_NAV,
  FILE_ACCESS_SETTINGS_NAV,
  type AgentLocalNavState,
  type AgentLocalWorkspaceState,
  type DeepPartial,
  type SettingsNavState,
} from "@/types/navigation";

interface ReadyAppProps {
  themeChoice: ThemeChoice;
  onThemeChange: (theme: ThemeChoice) => void;
  vaultError: boolean;
  onDismissVaultError: () => void;
  requestedExtensionId: string | null;
  onRequestedExtensionHandled: () => void;
}

export function ReadyApp(props: ReadyAppProps) {
  const {
    onDismissVaultError,
    onRequestedExtensionHandled,
    onThemeChange,
    requestedExtensionId,
    themeChoice,
    vaultError,
  } = props;
  const availability = useNavigationAvailability();
  const { current: nav, pushNav, replaceNav, goBack, goForward, canGoBack, canGoForward } =
    useTabHistory(DEFAULT_APP_NAV, availability);
  const { focusedPanel } = usePanelFocus();
  const { workspace, updateWorkspace, clearWorkspace } =
    useAgentSessionWorkspace(nav.agentLocal.sessionId);
  const agentNavState = useMemo<AgentLocalNavState>(() => ({
    sessionId: nav.agentLocal.sessionId,
    ...workspace,
  }), [nav.agentLocal.sessionId, workspace]);

  useEffect(() => {
    if (!requestedExtensionId) return;
    pushNav({
      tab: "settings",
      settings: {
        subTab: "extensions",
        extensionsSection: "custom",
        extensionId: requestedExtensionId,
      },
    });
    onRequestedExtensionHandled();
  }, [onRequestedExtensionHandled, pushNav, requestedExtensionId]);

  const onSettingsChange = useCallback((partial: DeepPartial<SettingsNavState>) => {
    pushNav({ settings: partial });
  }, [pushNav]);
  const onSettingsReplace = useCallback((partial: DeepPartial<SettingsNavState>) => {
    replaceNav({ settings: partial });
  }, [replaceNav]);
  const onWelcome = useCallback(() => {
    pushNav({ tab: "agent-local", agentLocal: { sessionId: null } });
  }, [pushNav]);
  const openFileAccessSettings = useCallback(() => {
    pushNav(FILE_ACCESS_SETTINGS_NAV);
  }, [pushNav]);
  const activeTab: TabId = nav.tab;

  useArrowNavigation({
    items: availability.mainTabs,
    selectedId: activeTab,
    onSelect: (tab) => pushNav({ tab }),
    enabled: focusedPanel === "sidebar",
    focusActiveSelector: "[data-nav-zone='sidebar'] [data-nav-active='true']",
  });

  return (
    <>
      {vaultError && <VaultErrorBanner onDismiss={onDismissVaultError} />}
      <AppNavigationActionsProvider openFileAccessSettings={openFileAccessSettings}>
        <AppLayout
          onOpenExtension={(extensionId) => pushNav({ tab: "settings", settings: { subTab: "extensions", extensionsSection: "custom", extensionId } })}
          activeTab={activeTab}
          onTabChange={(tab) => pushNav({ tab })}
          onShowWelcome={onWelcome}
          onBack={goBack}
          onForward={goForward}
          canGoBack={canGoBack}
          canGoForward={canGoForward}
          onSearchSelect={(sessionId) => pushNav({
            tab: "agent-local",
            agentLocal: { sessionId },
          })}
          onNewSession={onWelcome}
        >
          <CoreMainTabContent
            activeTab={activeTab}
            nav={nav}
            agentNavState={agentNavState}
            themeChoice={themeChoice}
            focusedPanel={focusedPanel}
            onWakeupChange={(wakeupId) => pushNav({ heartbeat: { wakeupId } })}
            onPathChange={(path) => pushNav({ personality: { path } })}
            onSessionChange={(sessionId) => pushNav({ agentLocal: { sessionId } })}
            onAgentNavChange={(partial: Partial<AgentLocalWorkspaceState>) => updateWorkspace(partial)}
            onWorkspaceClear={clearWorkspace}
            onThemeChange={onThemeChange}
            onSettingsNavChange={onSettingsChange}
            onSettingsNavReplace={onSettingsReplace}
          />
        </AppLayout>
      </AppNavigationActionsProvider>
    </>
  );
}
