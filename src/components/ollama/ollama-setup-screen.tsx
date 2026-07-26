import { useState, useCallback, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke, Channel } from "@tauri-apps/api/core";
import "./ollama.css";
import "./ollama-setup-screen.css";

interface OllamaSetupProgress {
  completed: number;
  total: number;
  status: string;
}

interface OllamaSetupScreenProps {
  onComplete: () => void | Promise<void>;
  onSkip?: () => void | Promise<void>;
}

export function OllamaSetupScreen({ onComplete, onSkip }: OllamaSetupScreenProps) {
  const { t } = useTranslation();
  const [downloading, setDownloading] = useState(false);
  const [cancelling, setCancelling] = useState(false);
  const [skipping, setSkipping] = useState(false);
  const [percent, setPercent] = useState(0);
  const [status, setStatus] = useState("");
  const [error, setError] = useState<string | null>(null);
  const cancelledRef = useRef(false);

  const isInstallPhase = useMemo(
    () => ["verifying", "extracting", "starting"].includes(status),
    [status],
  );

  const statusText = useMemo(() => {
    if (cancelling) return t("ollamaSetup.cancelling");
    if (status === "downloading-rocm") {
      return `${t("ollamaSetup.downloadingGpu")} ${percent}%`;
    }
    if (status === "downloading") {
      return `${t("ollamaSetup.downloading")} ${percent}%`;
    }
    if (status === "verifying") return t("ollamaSetup.verifying");
    if (status === "extracting") return t("ollamaSetup.extracting");
    if (status === "starting") return t("ollamaSetup.starting");
    return `${percent}%`;
  }, [cancelling, percent, status, t]);

  const handleDownload = useCallback(async () => {
    cancelledRef.current = false;
    setDownloading(true);
    setCancelling(false);
    setError(null);
    setPercent(0);
    setStatus("downloading");

    const channel = new Channel<OllamaSetupProgress>();
    channel.onmessage = (event) => {
      setStatus(event.status);
      if (event.total > 0) {
        setPercent(Math.round((event.completed / event.total) * 100));
      }
    };

    try {
      await invoke("download_ollama", { onProgress: channel });
      await onComplete();
    } catch {
      if (!cancelledRef.current) {
        setError(t("errors.operationFailed"));
      }
      setDownloading(false);
      setCancelling(false);
      setStatus("");
      setPercent(0);
    }
  }, [onComplete, t]);

  const handleCancel = useCallback(async () => {
    cancelledRef.current = true;
    setCancelling(true);
    setError(null);
    try {
      await invoke("cancel_ollama_setup");
      setDownloading(false);
      setCancelling(false);
      setStatus("");
      setPercent(0);
    } catch {
      cancelledRef.current = false;
      setCancelling(false);
      setError(t("errors.operationFailed"));
    }
  }, [t]);

  const handleSkip = useCallback(async () => {
    if (!onSkip) return;
    setSkipping(true);
    setError(null);
    try {
      await onSkip();
    } catch {
      setError(t("errors.operationFailed"));
      setSkipping(false);
    }
  }, [onSkip, t]);

  return (
    <div className="oss-container">
      <h2 className="oss-title">
        {t("ollamaSetup.title")}
      </h2>
      <p className="oss-description">
        {t("ollamaSetup.description")}
      </p>

      {downloading ? (
        <div className="oss-download-block">
          <div className="ollama-progress-bar oss-progress-bar">
            <div
              className={`ollama-progress-fill${isInstallPhase ? " oss-progress-fill-indeterminate" : ""}`}
              style={{ width: isInstallPhase ? "42%" : `${percent}%` }}
            />
          </div>
          <span className="oss-status-text">{statusText}</span>
          <button
            className="ollama-btn ollama-btn-primary oss-cancel-btn"
            onClick={() => void handleCancel()}
            disabled={cancelling}
          >
            {cancelling ? t("ollamaSetup.cancelling") : t("ollamaSetup.cancel")}
          </button>
        </div>
      ) : (
        <div className="oss-actions">
          <button
            className="ollama-btn ollama-btn-primary oss-download-btn"
            onClick={() => void handleDownload()}
            disabled={skipping}
          >
            {t("ollamaSetup.download")}
          </button>
          {onSkip && (
            <button
              className="ollama-btn oss-skip-btn"
              onClick={() => void handleSkip()}
              disabled={skipping}
            >
              {skipping ? t("ollamaSetup.skipping") : t("ollamaSetup.skip")}
            </button>
          )}
        </div>
      )}

      {error && (
        <div className="oss-error-block">
          <span className="oss-error-label">
            {t("ollamaSetup.error")}
          </span>
          <p className="oss-error-detail">
            {error}
          </p>
        </div>
      )}
    </div>
  );
}
