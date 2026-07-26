import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { showToast } from "@/lib/toast-emitter";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import i18n from "@/i18n";
import { useModelDownloads } from "@/hooks/use-model-downloads";
import { useForecastDevUpdates } from "@/hooks/use-forecast-dev-updates";

const CHECK_INTERVAL_MS = 60 * 60 * 1000;

export interface AppUpdate {
  version: string;
  assetUrl: string;
  title?: string | null;
  publishedAt?: string | null;
  notesByLocale?: Record<string, string[]> | null;
}

interface DownloadProgress {
  completed: number;
  total: number;
}

export interface OllamaModelUpdate {
  fullName: string;
  family: string;
  tag: string;
}

export interface OllamaBinaryUpdate {
  currentVersion: string;
  latestVersion: string;
}

export interface PullingState {
  fullName: string;
  percent: number;
  status: string;
}

export function useUpdateChecker() {
  const { activeDownload, startDownload } = useModelDownloads();
  const { forecastDevUpdates } = useForecastDevUpdates();
  const [appUpdate, setAppUpdate] = useState<AppUpdate | null>(null);
  const [ollamaUpdates, setOllamaUpdates] = useState<OllamaModelUpdate[]>([]);
  const [ollamaBinaryUpdate, setOllamaBinaryUpdate] =
    useState<OllamaBinaryUpdate | null>(null);
  const [ollamaBinaryUpdating, setOllamaBinaryUpdating] = useState(false);
  const [ollamaBinaryPercent, setOllamaBinaryPercent] = useState(0);
  const [appDownloading, setAppDownloading] = useState(false);
  const [appPercent, setAppPercent] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const checkAll = useCallback(async () => {
    try {
      const app = await invoke<AppUpdate | null>("check_app_update");
      setAppUpdate(app ?? null);
    } catch {
      /* network error, ignore */
    }

    try {
      const models = await invoke<OllamaModelUpdate[]>("check_ollama_updates");
      setOllamaUpdates(models);
    } catch {
      /* ollama not running, ignore */
    }

    try {
      const binary = await invoke<OllamaBinaryUpdate | null>(
        "check_ollama_binary_update",
      );
      setOllamaBinaryUpdate(binary ?? null);
    } catch {
      /* ollama not running or network error, ignore */
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- checkAll is async, setState in callbacks only
    void checkAll();
    timerRef.current = setInterval(() => void checkAll(), CHECK_INTERVAL_MS);
    const unlisten = listen("ollama-models-changed", () => {
      invoke<OllamaModelUpdate[]>("check_ollama_updates")
        .then(setOllamaUpdates)
        .catch(() => {});
    });
    return () => {
      clearInterval(timerRef.current);
      cleanupTauriListener(unlisten);
    };
  }, [checkAll]);

  const downloadAppUpdate = useCallback(async (assetUrl: string) => {
    setAppDownloading(true);
    setAppPercent(0);

    const channel = new Channel<DownloadProgress>();
    channel.onmessage = (event: DownloadProgress) => {
      const pct =
        event.total > 0 ? Math.round((event.completed / event.total) * 100) : 0;
      setAppPercent(pct);
    };

    try {
      await invoke("download_app_update", { assetUrl, onProgress: channel });
      setAppUpdate(null);
    } catch {
      showToast(i18n.t("errors.downloadFailed"), "error");
    } finally {
      setAppDownloading(false);
    }
  }, []);

  const pullModel = useCallback(async (fullName: string) => {
    try {
      await startDownload({
        kind: "ollama",
        modelId: fullName,
        isUpdate: true,
      });
    } catch {
      showToast(i18n.t("modelDownloads.errors.queueUnavailable"), "error");
    }
  }, [startDownload]);

  const updateOllamaBinary = useCallback(async () => {
    if (!ollamaBinaryUpdate) return;
    setOllamaBinaryUpdating(true);
    setOllamaBinaryPercent(0);

    interface SetupProgress {
      completed: number;
      total: number;
      status: string;
    }
    const channel = new Channel<SetupProgress>();
    channel.onmessage = (event: SetupProgress) => {
      if (event.status === "restarting") {
        setOllamaBinaryPercent(100);
        return;
      }
      if (event.total > 0) {
        setOllamaBinaryPercent(
          Math.round((event.completed / event.total) * 100),
        );
      }
    };

    try {
      await invoke("update_ollama_binary", {
        version: ollamaBinaryUpdate.latestVersion,
        onProgress: channel,
      });
      setOllamaBinaryUpdate(null);
    } catch {
      showToast(i18n.t("errors.updateFailed"), "error");
    } finally {
      setOllamaBinaryUpdating(false);
    }
  }, [ollamaBinaryUpdate]);

  const totalCount =
    (appUpdate ? 1 : 0) + (ollamaBinaryUpdate ? 1 : 0)
    + ollamaUpdates.length + forecastDevUpdates.length;

  const pulling = useMemo<PullingState | null>(() => {
    if (!activeDownload || activeDownload.kind !== "ollama") return null;
    if (!ollamaUpdates.some((update) => update.fullName === activeDownload.modelId)) {
      return null;
    }
    return {
      fullName: activeDownload.modelId,
      percent: activeDownload.percent,
      status: i18n.t(`modelDownloads.phases.${activeDownload.phase}`),
    };
  }, [activeDownload, ollamaUpdates]);

  return {
    appUpdate,
    ollamaUpdates,
    forecastDevUpdates,
    pulling,
    ollamaBinaryUpdate,
    ollamaBinaryUpdating,
    ollamaBinaryPercent,
    appDownloading,
    appPercent,
    totalCount,
    pullModel,
    downloadAppUpdate,
    updateOllamaBinary,
    checkAll,
  };
}
