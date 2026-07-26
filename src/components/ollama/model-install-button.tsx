import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { showToast } from "@/lib/toast-emitter";
import { useModelDownloads } from "@/hooks/use-model-downloads";
import "./ollama.css";

interface ModelInstallButtonProps {
  fullName: string;
  isInstalled: boolean;
  hasUpdate: boolean;
  sizeGb?: number;
}

const BTN_WIDTH = 88;

function isEscapeKey(e: KeyboardEvent): boolean {
  return e.code === "Escape";
}

export function ModelInstallButton(props: ModelInstallButtonProps) {
  const { fullName, isInstalled, hasUpdate, sizeGb } = props;
  const { t } = useTranslation();
  const { downloads, startDownload, cancelDownload } = useModelDownloads();
  const ownDownload = downloads.find(
    (item) => item.kind === "ollama"
      && item.modelId === fullName
      && (item.status === "running" || item.status === "queued"),
  ) ?? null;

  const handleInstall = useCallback(async () => {
    if (sizeGb && sizeGb > 0) {
      const sizeBytes = Math.round(sizeGb * 1_073_741_824);
      const fits = await invoke<boolean>("check_model_fits_vram", { sizeBytes }).catch(() => true);
      if (!fits) showToast(t("ollama.vramWarning"), "info", 4000);
    }
    try {
      await startDownload({
        kind: "ollama",
        modelId: fullName,
        isUpdate: isInstalled && hasUpdate,
      });
    } catch {
      showToast(t("modelDownloads.errors.queueUnavailable"), "info", 3000);
    }
  }, [fullName, hasUpdate, isInstalled, sizeGb, startDownload, t]);

  const handleCancel = useCallback(async () => {
    if (ownDownload) await cancelDownload(ownDownload.id).catch(() => {});
  }, [cancelDownload, ownDownload]);

  useEffect(() => {
    if (!ownDownload) return;
    const onEsc = (e: KeyboardEvent) => {
      if (isEscapeKey(e)) void handleCancel();
    };
    window.addEventListener("keydown", onEsc);
    return () => window.removeEventListener("keydown", onEsc);
  }, [handleCancel, ownDownload]);

  if (ownDownload) {
    const queued = ownDownload.status === "queued";
    return (
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 2 }}>
          {!queued && (
            <div className="ollama-progress-bar">
              <div className="ollama-progress-fill" style={{ width: `${ownDownload.percent}%` }} />
            </div>
          )}
          <span style={{ fontSize: "var(--text-xs)", color: "var(--ink-faint)" }}>
            {queued ? t("modelDownloads.queued") : t(`modelDownloads.phases.${ownDownload.phase}`)}
          </span>
        </div>
        <button
          className="ollama-btn ollama-btn-cancel"
          style={{ width: BTN_WIDTH }}
          onClick={() => void handleCancel()}
        >
          {t("ollama.cancel")}
        </button>
      </div>
    );
  }

  if (isInstalled && !hasUpdate) {
    return (
      <Tooltip label={t("ollama.installedUpToDate")}>
        <div style={{ display: "flex", alignItems: "center", color: "var(--select-text)" }}>
          <Check size="var(--icon-lg)" />
        </div>
      </Tooltip>
    );
  }

  const label = isInstalled && hasUpdate ? t("ollama.update") : t("ollama.install");

  const showUpdateTooltip = isInstalled && hasUpdate;
  const button = (
    <button
      className="ollama-btn ollama-btn-primary"
      style={{ width: BTN_WIDTH }}
      onClick={() => void handleInstall()}
    >
      {label}
    </button>
  );

  return showUpdateTooltip ? (
    <Tooltip label={t("ollama.updateAvailable")} align="right">
      {button}
    </Tooltip>
  ) : (
    button
  );
}
