import React from "react";
import ReactDOM from "react-dom/client";
import "@/i18n";
import { MascotOverlay } from "@/components/mascot/mascot-overlay";
import { ErrorBoundary } from "@/components/ui/error-boundary";
import { applyStoredSettings } from "@/hooks/use-settings";
import { installTauriListenerCleanupGuard } from "@/lib/tauri-listen";

installTauriListenerCleanupGuard();
applyStoredSettings();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary><MascotOverlay /></ErrorBoundary>
  </React.StrictMode>,
);
