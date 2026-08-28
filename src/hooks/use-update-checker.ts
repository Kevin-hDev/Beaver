import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useForecastDevUpdates } from "@/hooks/use-forecast-dev-updates";
import { useModelDownloads } from "@/hooks/use-model-downloads";
import { useUpdateDismissals } from "@/hooks/use-update-dismissals";
import { updateErrorKey } from "@/hooks/update-error";
import i18n from "@/i18n";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { showToast } from "@/lib/toast-emitter";

const CHECK_INTERVAL_MS = 60 * 60 * 1000;

export interface AppUpdate {
  version: string;
  assetUrl: string;
  title?: string | null;
  publishedAt?: string | null;
  notesByLocale?: Record<string, string[]> | null;
}

export interface OllamaModelUpdate {
  fullName: string;
  family: string;
  tag: string;
  latestDigest: string;
}

export interface OllamaBinaryUpdate {
  currentVersion: string;
  latestVersion: string;
}

export interface DismissedUpdate {
  kind: "app" | "ollama_binary" | "ollama_model";
  subject: string;
  version: string;
}

export interface PullingState {
  fullName: string;
  percent: number;
  status: string;
}

interface DownloadProgress {
  completed: number;
  total: number;
  status?: string;
}

export function useUpdateChecker() {
  const { activeDownload, startDownload, cancelDownload } = useModelDownloads();
  const { forecastDevUpdates } = useForecastDevUpdates();
  const dismissals = useUpdateDismissals();
  const [appUpdate, setAppUpdate] = useState<AppUpdate | null>(null);
  const [ollamaUpdates, setOllamaUpdates] = useState<OllamaModelUpdate[]>([]);
  const [ollamaBinaryUpdate, setOllamaBinaryUpdate] = useState<OllamaBinaryUpdate | null>(null);
  const [installedAppVersion, setInstalledAppVersion] = useState<string | null>(null);
  const [installedOllamaVersion, setInstalledOllamaVersion] = useState<string | null>(null);
  const [checking, setChecking] = useState(false);
  const [appDownloading, setAppDownloading] = useState(false);
  const [appPercent, setAppPercent] = useState(0);
  const [appCancelling, setAppCancelling] = useState(false);
  const [ollamaBinaryUpdating, setOllamaBinaryUpdating] = useState(false);
  const [ollamaBinaryPercent, setOllamaBinaryPercent] = useState(0);
  const [ollamaBinaryCancelling, setOllamaBinaryCancelling] = useState(false);
  const [modelCancellingId, setModelCancellingId] = useState<string | null>(null);
  const binaryBusy = useRef(false);
  const checkInFlight = useRef<Promise<void> | null>(null);
  const notifyCheckFailure = useRef(false);

  const checkAll = useCallback((notifyFailure = false) => {
    notifyCheckFailure.current ||= notifyFailure;
    if (checkInFlight.current) return checkInFlight.current;
    setChecking(true);
    const request = Promise.allSettled([
      invoke<AppUpdate | null>("check_app_update"),
      invoke<OllamaModelUpdate[]>("check_ollama_updates"),
      invoke<OllamaBinaryUpdate | null>("check_ollama_binary_update"),
      getVersion(),
      invoke<string | null>("get_ollama_installed_version"),
    ])
      .then((results) => {
        if (results[0].status === "fulfilled") {
          const discovered = results[0].value;
          setAppUpdate((known) => binaryBusy.current ? known : discovered);
        }
        if (results[1].status === "fulfilled") setOllamaUpdates(results[1].value);
        if (results[2].status === "fulfilled") {
          const discovered = results[2].value;
          setOllamaBinaryUpdate((known) => binaryBusy.current ? known : discovered);
        }
        if (results[3].status === "fulfilled") setInstalledAppVersion(results[3].value);
        if (results[4].status === "fulfilled") setInstalledOllamaVersion(results[4].value);
        if (notifyCheckFailure.current && results[2].status === "rejected") {
          showToast(i18n.t("updates.checkFailed"), "error");
        }
      })
      .finally(() => {
        checkInFlight.current = null;
        notifyCheckFailure.current = false;
        setChecking(false);
      });
    checkInFlight.current = request;
    return request;
  }, []);

  useEffect(() => {
    void checkAll();
    const timer = setInterval(() => void checkAll(), CHECK_INTERVAL_MS);
    const unlisten = listen("ollama-models-changed", () => {
      invoke<OllamaModelUpdate[]>("check_ollama_updates").then(setOllamaUpdates).catch(() => {});
    });
    return () => {
      clearInterval(timer);
      cleanupTauriListener(unlisten);
    };
  }, [checkAll]);

  const downloadAppUpdate = useCallback(async (assetUrl: string) => {
    if (binaryBusy.current) return;
    binaryBusy.current = true;
    setAppCancelling(false);
    setAppDownloading(true);
    setAppPercent(0);
    const channel = new Channel<DownloadProgress>();
    channel.onmessage = ({ completed, total }) => setAppPercent(total > 0 ? Math.round(completed / total * 100) : 0);
    try {
      await invoke("download_app_update", { assetUrl, onProgress: channel });
      setAppUpdate(null);
    } catch (error) {
      const cancelled = isError(error, "update-download-cancelled");
      showToast(i18n.t(cancelled ? "updates.cancelled" : updateErrorKey(error)), cancelled ? "success" : "error");
    } finally {
      binaryBusy.current = false;
      setAppCancelling(false);
      setAppDownloading(false);
    }
  }, []);

  const updateOllamaBinary = useCallback(async () => {
    if (!ollamaBinaryUpdate || binaryBusy.current) return;
    binaryBusy.current = true;
    setOllamaBinaryCancelling(false);
    setOllamaBinaryUpdating(true);
    setOllamaBinaryPercent(0);
    const channel = new Channel<DownloadProgress>();
    channel.onmessage = ({ completed, total, status }) => setOllamaBinaryPercent(status === "restarting" ? 100 : total > 0 ? Math.round(completed / total * 100) : 0);
    try {
      await invoke("update_ollama_binary", { version: ollamaBinaryUpdate.latestVersion, onProgress: channel });
      setInstalledOllamaVersion(ollamaBinaryUpdate.latestVersion);
      setOllamaBinaryUpdate(null);
    } catch (error) {
      const cancelled = isError(error, "ollama-operation-cancelled");
      showToast(i18n.t(cancelled ? "updates.cancelled" : "errors.updateFailed"), cancelled ? "success" : "error");
    } finally {
      binaryBusy.current = false;
      setOllamaBinaryCancelling(false);
      setOllamaBinaryUpdating(false);
    }
  }, [ollamaBinaryUpdate]);

  const pullModel = useCallback(async (fullName: string) => {
    try {
      await startDownload({ kind: "ollama", modelId: fullName, isUpdate: true });
    } catch {
      showToast(i18n.t("modelDownloads.errors.queueUnavailable"), "error");
    }
  }, [startDownload]);

  const cancelAppUpdate = useCallback(async () => {
    setAppCancelling(true);
    try {
      await invoke("cancel_app_update_download");
    } catch {
      // Le téléchargement reste affiché et peut être annulé à nouveau.
      setAppCancelling(false);
      return;
    }
    if (!binaryBusy.current) setAppCancelling(false);
  }, []);
  const cancelOllamaBinary = useCallback(async () => {
    setOllamaBinaryCancelling(true);
    try {
      await invoke("cancel_ollama_setup");
    } catch {
      // Le téléchargement reste affiché et peut être annulé à nouveau.
      setOllamaBinaryCancelling(false);
      return;
    }
    if (!binaryBusy.current) setOllamaBinaryCancelling(false);
  }, []);
  const cancelModelUpdate = useCallback(async () => {
    if (!activeDownload || activeDownload.kind !== "ollama") return;
    setModelCancellingId(activeDownload.id);
    await cancelDownload(activeDownload.id).catch(() => setModelCancellingId(null));
  }, [activeDownload, cancelDownload]);

  const visibleAppUpdate = dismissals.visible(appUpdate, (update) => ({ kind: "app", subject: "beaver", version: update.version }));
  const visibleOllamaBinaryUpdate = dismissals.visible(ollamaBinaryUpdate, (update) => ({ kind: "ollama_binary", subject: "ollama", version: update.latestVersion }));
  const visibleOllamaUpdates = dismissals.filter(ollamaUpdates, (update) => ({ kind: "ollama_model", subject: update.fullName, version: update.latestDigest }));
  const pulling = useMemo<PullingState | null>(() => {
    if (!activeDownload || activeDownload.kind !== "ollama" || !ollamaUpdates.some((update) => update.fullName === activeDownload.modelId)) return null;
    return { fullName: activeDownload.modelId, percent: activeDownload.percent, status: i18n.t(`modelDownloads.phases.${activeDownload.phase}`) };
  }, [activeDownload, ollamaUpdates]);

  return {
    appUpdate, ollamaBinaryUpdate, ollamaUpdates, installedAppVersion, installedOllamaVersion,
    visibleAppUpdate, visibleOllamaBinaryUpdate, visibleOllamaUpdates, forecastDevUpdates,
    checking, pulling, appDownloading, appPercent, appCancelling,
    ollamaBinaryUpdating, ollamaBinaryPercent, ollamaBinaryCancelling,
    modelCancelling: modelCancellingId !== null && modelCancellingId === activeDownload?.id,
    binaryBusy: appDownloading || ollamaBinaryUpdating,
    totalCount: (visibleAppUpdate ? 1 : 0) + (visibleOllamaBinaryUpdate ? 1 : 0) + visibleOllamaUpdates.length + forecastDevUpdates.length,
    checkAll, pullModel, downloadAppUpdate, updateOllamaBinary,
    cancelAppUpdate, cancelOllamaBinary, cancelModelUpdate,
    dismissUpdate: dismissals.dismiss,
  };
}

function isError(error: unknown, code: string): boolean {
  return typeof error === "string" && error.includes(code);
}
