import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { StartupWindowControls } from "@/components/layout/startup-window-controls";
import { ReadyApp } from "@/components/layout/ready-app";
import { OllamaSetupScreen } from "@/components/ollama/ollama-setup-screen";
import { OnboardingScreen } from "@/components/onboarding/onboarding-screen";
import { useTheme } from "@/hooks/use-theme";
import { ForecastDocsWindow } from "@/components/forecast-docs/forecast-docs-window";
import { ForecastWorkbenchApp } from "@/components/forecast/workbench/forecast-workbench-app";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { useStartupGate } from "@/hooks/use-startup-gate";
import { ExtensionsProvider } from "@/hooks/use-extensions";
import { ExtensionUiStartupBoundary } from "@/components/extensions/extension-ui-startup-boundary";
import { NORMAL_EXTENSION_UI_STARTUP } from "@/lib/extension-ui-startup";
import type { ExtensionUiStartupState } from "@/types/extensions";
import { usePlatformBodyClass } from "@/hooks/use-platform-body-class";
import { useBrowserRecoveryNotice } from "@/hooks/use-browser-recovery-notice";
import { UpdateProvider } from "@/hooks/update-context";
import { SlotProvider } from "@/features/extension-ui/slot-provider";
import { StandardCatalogProvider } from "@/features/extension-ui/standard/catalog-context";
import "./App.css";

export default function App({ initialExtensionUiStartup = NORMAL_EXTENSION_UI_STARTUP }:
{ initialExtensionUiStartup?: ExtensionUiStartupState }) {
  usePlatformBodyClass();

  if (window.location.hash === "#/forecast-docs") return <ForecastDocsApp />;
  if (window.location.hash === "#/forecast-workbench") return <ForecastWorkbenchApp />;
  return <MainApp initialExtensionUiStartup={initialExtensionUiStartup} />;
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
  const { choice, setTheme } = useTheme();
  const [vaultError, setVaultError] = useState(false);
  const [requestedExtensionId, setRequestedExtensionId] = useState<string | null>(null);
  const startupGate = useStartupGate();

  useEffect(() => {
    const unlisten = listen<void>("vault-init-failed", () => {
      setVaultError(true);
    });
    return () => { cleanupTauriListener(unlisten); };
  }, []);

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
      onOpenExtension={setRequestedExtensionId}
    >
      <ExtensionsProvider>
        <UpdateProvider>
          <StandardCatalogProvider onOpenExtension={setRequestedExtensionId}>
            <SlotProvider>
              <ReadyApp
                themeChoice={choice}
                onThemeChange={setTheme}
                vaultError={vaultError}
                onDismissVaultError={() => setVaultError(false)}
                requestedExtensionId={requestedExtensionId}
                onRequestedExtensionHandled={() => setRequestedExtensionId(null)}
              />
            </SlotProvider>
          </StandardCatalogProvider>
        </UpdateProvider>
      </ExtensionsProvider>
    </ExtensionUiStartupBoundary>
  );
}
