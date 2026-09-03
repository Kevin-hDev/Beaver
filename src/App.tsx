import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { AppLayout } from "@/components/layout/app-layout";
import { StartupWindowControls } from "@/components/layout/startup-window-controls";
import { VaultErrorBanner } from "@/components/layout/vault-error-banner";
import { OllamaSetupScreen } from "@/components/ollama/ollama-setup-screen";
import { OnboardingScreen } from "@/components/onboarding/onboarding-screen";
import { useTheme } from "@/hooks/use-theme";
import { useTabHistory } from "@/hooks/use-tab-history";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import { usePanelFocus } from "@/hooks/use-panel-focus";
import { CoreMainTabContent } from "@/components/layout/core-main-tab-content";
import { ForecastDocsWindow } from "@/components/forecast-docs/forecast-docs-window";
import { ForecastWorkbenchApp } from "@/components/forecast/workbench/forecast-workbench-app";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { useStartupGate } from "@/hooks/use-startup-gate";
import { ExtensionsProvider } from "@/hooks/use-extensions";
import { ExtensionUiStartupBoundary } from "@/components/extensions/extension-ui-startup-boundary";
import { NORMAL_EXTENSION_UI_STARTUP } from "@/lib/extension-ui-startup";
import type { ExtensionUiStartupState } from "@/types/extensions";
import { usePlatformBodyClass } from "@/hooks/use-platform-body-class";
import { AppNavigationActionsProvider } from "@/hooks/use-app-navigation-actions";
import { useBrowserRecoveryNotice } from "@/hooks/use-browser-recovery-notice";
import { useAgentSessionWorkspace } from "@/hooks/use-agent-session-workspace";
import { UpdateProvider } from "@/hooks/update-context";
import type { TabId } from "@/components/layout/nav-items";
import { SlotProvider } from "@/features/extension-ui/slot-provider";
import { useNavigationAvailability } from "@/features/extension-ui/slot-contexts";
import "./App.css";
import {
  DEFAULT_APP_NAV,
  FILE_ACCESS_SETTINGS_NAV,
  type AgentLocalNavState,
  type AgentLocalWorkspaceState,
  type DeepPartial,
  type SettingsNavState,
} from "@/types/navigation";

export default function App({ initialExtensionUiStartup = NORMAL_EXTENSION_UI_STARTUP }:
{ initialExtensionUiStartup?: ExtensionUiStartupState }) {
  usePlatformBodyClass();

  if (window.location.hash === "#/forecast-docs") return <ForecastDocsApp />;
  if (window.location.hash === "#/forecast-workbench") return <ForecastWorkbenchApp />;
  return (
    <SlotProvider>
      <MainApp initialExtensionUiStartup={initialExtensionUiStartup} />
    </SlotProvider>
  );
}

function ForecastDocsApp() {
  useTheme();

  useEffect(() => {
    const splash = document.getElementById("splash");
    if (!splash) return;
    requestAnimationFrame(() => splash.remove());
  }, []);

  return <ForecastDocsWindow />;
}

function MainApp({ initialExtensionUiStartup }: { initialExtensionUiStartup: ExtensionUiStartupState }) {
  useBrowserRecoveryNotice();
  const navigationAvailability = useNavigationAvailability();
  const { current: nav, pushNav, replaceNav, goBack, goForward, canGoBack, canGoForward } =
    useTabHistory(DEFAULT_APP_NAV, navigationAvailability);

  const { choice, setTheme } = useTheme();
  const [vaultError, setVaultError] = useState(false);
  const { focusedPanel } = usePanelFocus();
  const startupGate = useStartupGate();

  useEffect(() => {
    const unlisten = listen<void>("vault-init-failed", () => {
      setVaultError(true);
    });
    return () => { cleanupTauriListener(unlisten); };
  }, []);

  const activeTab: TabId = nav.tab;
  const {
    workspace: agentWorkspace,
    updateWorkspace: updateAgentWorkspace,
    clearWorkspace: clearAgentWorkspace,
  } =
    useAgentSessionWorkspace(nav.agentLocal.sessionId);
  const agentNavState = useMemo<AgentLocalNavState>(() => ({
    sessionId: nav.agentLocal.sessionId,
    ...agentWorkspace,
  }), [agentWorkspace, nav.agentLocal.sessionId]);
  const handleWakeupChange = useCallback((id: string | null) => pushNav({ heartbeat: { wakeupId: id } }), [pushNav]);
  const handlePathChange = useCallback((path: string | null) => pushNav({ personality: { path } }), [pushNav]);
  const handleSessionChange = useCallback((id: string | null) => pushNav({ agentLocal: { sessionId: id } }), [pushNav]);
  const handleAgentNavChange = useCallback((partial: Partial<AgentLocalWorkspaceState>) => {
    updateAgentWorkspace(partial);
  }, [updateAgentWorkspace]);
  const handleSettingsNavChange = useCallback((partial: DeepPartial<SettingsNavState>) => {
    pushNav({ settings: partial });
  }, [pushNav]);
  const handleSettingsNavReplace = useCallback((partial: DeepPartial<SettingsNavState>) => {
    replaceNav({ settings: partial });
  }, [replaceNav]);
  const openFileAccessSettings = useCallback(() => {
    pushNav(FILE_ACCESS_SETTINGS_NAV);
  }, [pushNav]);

  useArrowNavigation({
    items: navigationAvailability.mainTabs,
    selectedId: activeTab,
    onSelect: (t) => pushNav({ tab: t }),
    enabled: focusedPanel === "sidebar",
    focusActiveSelector: "[data-nav-zone='sidebar'] [data-nav-active='true']",
  });

  const handleShowWelcome = useCallback(() => {
    pushNav({ tab: "agent-local", agentLocal: { sessionId: null } });
  }, [pushNav]);

  const handleSearchSelect = useCallback((sessionId: string) => {
    pushNav({ tab: "agent-local", agentLocal: { sessionId } });
  }, [pushNav]);

  useEffect(() => {
    if (startupGate.view === "loading") return;
    const timer = setTimeout(() => {
      requestAnimationFrame(() => {
        document.getElementById("splash")?.remove();
      });
    }, 150);
    return () => clearTimeout(timer);
  }, [startupGate.view]);

  /* Le splash couvre encore la fenêtre pendant tout ce temps : les boutons sont
     le seul moyen de la fermer ou de la réduire là où les décorations natives
     ont été retirées. */
  if (startupGate.view === "loading") {
    return <StartupWindowControls />;
  }

  if (startupGate.view === "onboarding") {
    return (
      <OnboardingScreen
        themeChoice={choice}
        onThemeChange={setTheme}
        showOllamaStep={startupGate.showOllamaSetup}
        onCompleteOnboarding={startupGate.completeOnboarding}
        onCompleteOllama={startupGate.completeOllamaSetup}
        onSkipOllama={startupGate.skipOllamaSetup}
      />
    );
  }

  if (startupGate.view === "ollama") {
    return (
      <div className="app-startup-shell">
        <StartupWindowControls />
        <OllamaSetupScreen
          onComplete={startupGate.completeOllamaSetup}
          onSkip={startupGate.skipOllamaSetup}
        />
      </div>
    );
  }

  return (
    <ExtensionUiStartupBoundary
      initial={initialExtensionUiStartup}
      onOpenExtension={(extensionId) => pushNav({
        tab: "settings",
        settings: { subTab: "extensions", extensionsSection: "custom", extensionId },
      })}
    >
    <ExtensionsProvider>
      <UpdateProvider>
        {vaultError && <VaultErrorBanner onDismiss={() => setVaultError(false)} />}
        <AppNavigationActionsProvider openFileAccessSettings={openFileAccessSettings}>
        <AppLayout
          activeTab={activeTab}
          onTabChange={(t) => pushNav({ tab: t })}
          onShowWelcome={handleShowWelcome}
          onBack={goBack}
          onForward={goForward}
          canGoBack={canGoBack}
          canGoForward={canGoForward}
          onSearchSelect={handleSearchSelect}
          onNewSession={handleShowWelcome}
        >
          <CoreMainTabContent
            activeTab={activeTab}
            nav={nav}
            agentNavState={agentNavState}
            themeChoice={choice}
            focusedPanel={focusedPanel}
            onWakeupChange={handleWakeupChange}
            onPathChange={handlePathChange}
            onSessionChange={handleSessionChange}
            onAgentNavChange={handleAgentNavChange}
            onWorkspaceClear={clearAgentWorkspace}
            onThemeChange={setTheme}
            onSettingsNavChange={handleSettingsNavChange}
            onSettingsNavReplace={handleSettingsNavReplace}
          />
        </AppLayout>
        </AppNavigationActionsProvider>
      </UpdateProvider>
    </ExtensionsProvider>
    </ExtensionUiStartupBoundary>
  );
}
