import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import { ConfirmButton } from "@/components/settings/confirm-button";
import { Tooltip } from "@/components/ui/tooltip";
import { useModelDownloads } from "@/hooks/use-model-downloads";
import { showToast } from "@/lib/toast-emitter";
import "../../ollama/ollama.css";
import "./model-install-btn.css";

interface ModelInstallBtnProps {
  modelId: string;
  installed: boolean;
  runtimeReady: boolean;
  allowUninstall?: boolean;
  onDone: () => void;
}

export function ModelInstallBtn({
  modelId,
  installed,
  runtimeReady,
  allowUninstall = false,
  onDone,
}: ModelInstallBtnProps) {
  const { t } = useTranslation();
  const [uninstalling, setUninstalling] = useState(false);
  const [startFailed, setStartFailed] = useState(false);
  const { startDownload, cancelDownload, downloads } = useModelDownloads();
  const ownDownload = downloads.find(
    (item) => item.kind === "forecast"
      && item.modelId === modelId
      && (item.status === "running" || item.status === "queued"),
  ) ?? null;
  const hasPendingDownloads = downloads.some(
    (item) => item.status === "running" || item.status === "queued",
  );
  const finishedOwn = downloads.find(
    (item) => item.kind === "forecast" && item.modelId === modelId && item.status === "completed",
  );
  const failedOwn = downloads.find(
    (item) => item.kind === "forecast" && item.modelId === modelId && item.status === "failed",
  );

  useEffect(() => {
    if (finishedOwn) onDone();
  }, [finishedOwn, onDone]);

  const handleInstall = useCallback(async () => {
    setStartFailed(false);
    try {
      await startDownload({ kind: "forecast", modelId });
    } catch {
      setStartFailed(true);
    }
  }, [modelId, startDownload]);

  const handleCancel = useCallback(async () => {
    if (ownDownload) await cancelDownload(ownDownload.id).catch(() => {});
  }, [cancelDownload, ownDownload]);

  const handleUninstall = useCallback(async () => {
    setUninstalling(true);
    try {
      await invoke("uninstall_forecast_model", { name: modelId });
      onDone();
    } catch {
      showToast(t("errors.operationFailed"), "error");
    } finally {
      setUninstalling(false);
    }
  }, [modelId, onDone, t]);

  useEffect(() => {
    if (!ownDownload) return;
    const onEsc = (e: KeyboardEvent) => {
      if (e.code === "Escape") void handleCancel();
    };
    window.addEventListener("keydown", onEsc);
    return () => window.removeEventListener("keydown", onEsc);
  }, [handleCancel, ownDownload]);

  if (ownDownload) {
    const queued = ownDownload.status === "queued";
    return (
      <div className="fmi-progress">
        <span className="fmi-phase">
          {queued ? t("modelDownloads.queued") : t(`modelDownloads.phases.${ownDownload.phase}`)}
        </span>
        {!queued && (
          <>
            <div className="fmi-bar">
              <div className="fmi-fill" style={{ transform: `scaleX(${ownDownload.percent / 100})` }} />
            </div>
            <span className="fmi-pct">{ownDownload.percent}%</span>
          </>
        )}
        <button className="ollama-btn fmi-cancel" onClick={() => void handleCancel()}>
          {t("forecast.models.cancel")}
        </button>
      </div>
    );
  }

  const downloadAction = (
    <div className="fmi-action">
      <button
        className="ollama-btn fmi-btn"
        onClick={() => void handleInstall()}
      >
        {installed ? t("forecast.models.prepare") : t("forecast.models.install")}
      </button>
      {failedOwn && (
        <span className="fmi-error" role="alert">{t("errors.downloadFailed")}</span>
      )}
      {startFailed && (
        <span className="fmi-error" role="alert">
          {t("modelDownloads.errors.queueUnavailable")}
        </span>
      )}
    </div>
  );

  if (installed && allowUninstall) {
    return (
      <div className="fmi-actions">
        {!runtimeReady && downloadAction}
        <ConfirmButton
          className="ollama-btn fmi-btn fmi-uninstall"
          label={uninstalling ? t("forecast.models.uninstalling") : t("forecast.models.uninstall")}
          confirmLabel={t("forecast.models.confirmUninstall")}
          onConfirm={() => void handleUninstall()}
          disabled={uninstalling || hasPendingDownloads}
        />
      </div>
    );
  }

  if (installed && runtimeReady) {
    return (
      <Tooltip label={t("forecast.models.installed")}>
        <div className="fmi-installed">
          <Check size="var(--icon-lg)" />
        </div>
      </Tooltip>
    );
  }

  return downloadAction;
}
