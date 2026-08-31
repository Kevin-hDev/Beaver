import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useOllamaModels } from "@/hooks/use-ollama-models";
import { useOllamaRuntimeStatus } from "@/hooks/use-ollama-runtime-status";
import { ollamaErrorKey, ollamaProgressKey } from "@/lib/ollama-runtime-error";
import { ModelfileIcon, ModelsIcon } from "@/components/ui/model-browser-icons";
import { SettingsPanel } from "@/components/settings/shell/settings-panel";
import { SettingsTabbar } from "@/components/settings/shell/settings-tabbar";
import { ollamaSetupSkippedPatch } from "@/lib/ollama-setup-gate";
import { OllamaSetupScreen } from "./ollama-setup-screen";
import { OllamaModelfileView } from "./ollama-modelfile-view";
import { OllamaModelsView } from "./ollama-models-view";
import type { RegistryModel } from "@/types/agent";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import type { DaemonState, OllamaRuntimeStatus } from "@/types/ollama-runtime";
import "./ollama.css";

interface OllamaTabProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

export function useOllamaTabContent({ navState, onNavChange, onNavReplace }: OllamaTabProps): React.ReactNode {
  const { t } = useTranslation();
  const subTab = navState.ollamaSubTab;
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<RegistryModel[]>([]);
  const [searching, setSearching] = useState(false);
  const runtime = useOllamaRuntimeStatus();
  const { loading: runtimeLoading, readError: runtimeReadError, status: runtimeStatus, refresh: refreshRuntime } = runtime;
  const ollamaModels = useOllamaModels({ enabled: runtimeStatus?.bundle === "ready" });

  const tabs = useMemo(() => [
    { id: "modelfile" as const, label: t("ollama.modelfileTab"), icon: <ModelfileIcon size="var(--icon-md)" /> },
    { id: "models" as const, label: t("ollama.modelsTab"), icon: <ModelsIcon size="var(--icon-md)" /> },
  ], [t]);

  const search = useMemo(() => ({
    query: searchQuery,
    setQuery: setSearchQuery,
    results: searchResults,
    setResults: setSearchResults,
    searching,
    setSearching,
  }), [searchQuery, searchResults, searching]);

  const detail = useMemo(() => {
    if (runtimeLoading) {
      return <RuntimeMessage message={t("ollama.runtime.loading")} />;
    }
    if (runtimeReadError || !runtimeStatus) {
      return <RuntimeRetry error={null} onRetry={refreshRuntime} t={t} />;
    }
    if (runtimeStatus.bundle === "absent") {
      return (
        <div className="ollama-setup-detail">
          <OllamaSetupScreen
            onComplete={async () => {
              await invoke("patch_advanced_settings", { patch: ollamaSetupSkippedPatch(false) });
              await refreshRuntime();
            }}
          />
        </div>
      );
    }
    if (runtimeStatus.bundle === "transaction_pending") {
      return <RuntimeMessage message={progressLabel(runtimeStatus, t)} />;
    }
    if (runtimeStatus.bundle === "recovery_required") {
      return (
        <RuntimeRetry
          error={runtimeStatus.last_error}
          onRetry={retryRecovery(refreshRuntime)}
          t={t}
        />
      );
    }
    return (
      <SettingsPanel title={t("settings.tabs.ollama")}>
        <OllamaDaemonNotice daemon={daemonKind(runtimeStatus.daemon)} t={t} />
        <SettingsTabbar
          items={tabs}
          active={subTab}
          label={t("settings.tabs.ollama")}
          onChange={(ollamaSubTab) => onNavChange({ ollamaSubTab })}
        />
        {subTab === "modelfile" ? (
          <OllamaModelfileView
            models={ollamaModels.models}
            selected={navState.ollamaInstalledModel}
            onSelect={(ollamaInstalledModel) => onNavReplace({ ollamaInstalledModel })}
          />
        ) : (
          <OllamaModelsView
            search={search}
            family={navState.ollamaFamily}
            variant={navState.ollamaVariant}
            onSelectFamily={(ollamaFamily) => onNavReplace({ ollamaFamily, ollamaVariant: null })}
            onSelectVariant={(ollamaVariant) => onNavReplace({ ollamaVariant })}
          />
        )}
      </SettingsPanel>
    );
  }, [
    navState.ollamaFamily,
    navState.ollamaInstalledModel,
    navState.ollamaVariant,
    runtimeLoading,
    runtimeReadError,
    refreshRuntime,
    runtimeStatus,
    ollamaModels.models,
    onNavChange,
    onNavReplace,
    search,
    subTab,
    t,
    tabs,
  ]);

  return detail;
}

function RuntimeMessage({ message }: { message: string }) {
  return <div className="ollama-runtime-message">{message}</div>;
}

function RuntimeRetry({
  error, onRetry, t,
}: { error: unknown; onRetry: () => Promise<void>; t: (key: string) => string }) {
  const [retrying, setRetrying] = useState(false);
  const retry = async () => {
    setRetrying(true);
    try { await onRetry(); } finally { setRetrying(false); }
  };
  return (
    <div className="ollama-runtime-message" data-ollama-runtime-error="generic">
      <p>{t(ollamaErrorKey(error))}</p>
      <button className="btn btn-sm btn-primary" onClick={() => void retry()} disabled={retrying}>
        {t("ollama.runtime.retry")}
      </button>
    </div>
  );
}

function retryRecovery(refresh: () => Promise<void>): () => Promise<void> {
  return async () => {
    try {
      await invoke("retry_ollama_recovery");
    } catch {
      // The status panel remains generic; backend details never reach the UI.
    }
    await refresh();
  };
}

function progressLabel(status: OllamaRuntimeStatus, t: (key: string) => string): string {
  if (status.operation === "cancelling") return t("ollamaSetup.cancelling");
  if (status.progress === null) return t("ollama.runtime.loading");
  return t(ollamaProgressKey(status.progress));
}

/**
 * N'annonce l'état du démon que lorsqu'il apprend quelque chose. « Géré par
 * Beaver » est le cas normal : la ligne n'y disait rien et occupait un rang
 * entier au-dessus des onglets. Ollama externe et Ollama indisponible, eux,
 * expliquent pourquoi la liste des modèles n'est pas celle attendue.
 */
function OllamaDaemonNotice({
  daemon,
  t,
}: {
  daemon: "owned" | "external" | "unavailable";
  t: (key: string) => string;
}) {
  if (daemon === "owned") return null;
  return (
    <p className="settings-panel-description" data-ollama-daemon={daemon}>
      {t(`ollama.runtime.${daemon}`)}
    </p>
  );
}

function daemonKind(daemon: DaemonState | null | undefined): "owned" | "external" | "unavailable" {
  if (!daemon || typeof daemon === "string") return "unavailable";
  if ("external" in daemon) return "external";
  return "owned";
}
