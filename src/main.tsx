import React from "react";
import ReactDOM from "react-dom/client";
import "@/i18n";
import "@/styles/global.css";
import { ErrorBoundary } from "@/components/ui/error-boundary";
import { ToastProvider } from "@/components/ui/toast";
import { installTauriListenerCleanupGuard } from "@/lib/tauri-listen";
import { applyStoredSettings } from "@/hooks/use-settings";
import App from "./App";

installTauriListenerCleanupGuard();
applyStoredSettings();

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <ToastProvider>
        <App />
      </ToastProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);
