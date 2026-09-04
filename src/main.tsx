import React from "react";
import ReactDOM from "react-dom/client";
import "@/i18n";
import "@/styles/global.css";
import { ErrorBoundary } from "@/components/ui/error-boundary";
import { ToastProvider } from "@/components/ui/toast";
import { installTauriListenerCleanupGuard } from "@/lib/tauri-listen";
import { applyStoredSettings } from "@/hooks/use-settings";
import { BrowserCapabilityProvider } from "@/hooks/use-browser-capability";
import App from "./App";
import {
  FAIL_CLOSED_EXTENSION_UI_STARTUP,
  installExtensionUiStartupCapture,
} from "@/lib/extension-ui-startup";

// Installed at module start, before React can mount any provider or extension surface.
const extensionUiStartup = installExtensionUiStartupCapture()
  .catch(() => FAIL_CLOSED_EXTENSION_UI_STARTUP);

async function startApplication() {
  const initialExtensionUiStartup = await extensionUiStartup;
  if (import.meta.env.VITE_E2E === "1") {
    await import("@wdio/tauri-plugin");
  }

  installTauriListenerCleanupGuard();
  applyStoredSettings();

  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <ErrorBoundary>
        <ToastProvider>
          <BrowserCapabilityProvider>
            <App initialExtensionUiStartup={initialExtensionUiStartup} />
          </BrowserCapabilityProvider>
        </ToastProvider>
      </ErrorBoundary>
    </React.StrictMode>,
  );
}

void startApplication();
