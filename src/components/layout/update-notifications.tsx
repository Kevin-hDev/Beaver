import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { AppUpdate, OllamaModelUpdate, OllamaBinaryUpdate, PullingState } from "@/hooks/use-update-checker";
import type { ForecastDevUpdate } from "@/hooks/use-forecast-dev-updates";
import { BRAND } from "@/lib/brand";
import { BubbleItem, type ItemData } from "./bubble-item";
import "./update-notifications.css";
import "./update-notifications-controls.css";

interface UpdateNotificationsProps {
  isOpen: boolean;
  onClose: () => void;
  appUpdate: AppUpdate | null;
  ollamaBinaryUpdate: OllamaBinaryUpdate | null;
  ollamaUpdates: OllamaModelUpdate[];
  forecastDevUpdates: ForecastDevUpdate[];
  pulling: PullingState | null;
  ollamaBinaryUpdating: boolean;
  ollamaBinaryPercent: number;
  appDownloading: boolean;
  appPercent: number;
  onPullModel: (fullName: string) => void;
  onDownloadApp: (dmgUrl: string) => void;
  onUpdateOllamaBinary: () => void;
  anchorLeft: number;
}

export function UpdateNotifications({
  isOpen, onClose,
  appUpdate, ollamaBinaryUpdate, ollamaUpdates, forecastDevUpdates,
  pulling, ollamaBinaryUpdating, ollamaBinaryPercent,
  appDownloading, appPercent,
  onPullModel, onDownloadApp, onUpdateOllamaBinary,
  anchorLeft,
}: UpdateNotificationsProps) {
  const { t, i18n } = useTranslation();
  const [closing, setClosing] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);

  const items = buildItems(
    t, i18n.language, appUpdate, ollamaBinaryUpdate, ollamaUpdates, forecastDevUpdates,
  );
  const maxDelay = items.length * 80;
  const closeDelay = maxDelay + 400;

  const handleClose = useCallback(() => {
    setClosing(true);
    setTimeout(() => {
      setClosing(false);
      onClose();
    }, closeDelay);
  }, [onClose, closeDelay]);

  useEffect(() => {
    if (!isOpen) return;
    const onEscape = (e: KeyboardEvent) => {
      if (e.code === "Escape") handleClose();
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  }, [isOpen, handleClose]);

  if (!isOpen) return null;

  return (
    <>
      <div className="update-overlay" role="presentation" onClick={handleClose} onKeyDown={() => {}} />
      <div ref={listRef} className="update-list" style={{ left: anchorLeft }}>
        {items.map((item, i) => (
          <BubbleItem
            key={item.id}
            item={item}
            index={i}
            closing={closing}
            totalCount={items.length}
            pulling={pulling}
            ollamaBinaryUpdating={ollamaBinaryUpdating}
            ollamaBinaryPercent={ollamaBinaryPercent}
            appDownloading={appDownloading}
            appPercent={appPercent}
            onPullModel={onPullModel}
            onDownloadApp={onDownloadApp}
            onUpdateOllamaBinary={onUpdateOllamaBinary}
            t={t}
          />
        ))}
      </div>
    </>
  );
}

function buildItems(
  t: (k: string, opts?: Record<string, string>) => string,
  language: string,
  app: AppUpdate | null,
  binary: OllamaBinaryUpdate | null,
  models: OllamaModelUpdate[],
  forecastUpdates: ForecastDevUpdate[],
): ItemData[] {
  const items: ItemData[] = [];
  if (app) {
    items.push({
      id: "app",
      type: "app",
      name: BRAND.displayName,
      sub: t("updates.version", { version: app.version }),
      version: app.version,
      title: app.title,
      publishedAt: app.publishedAt,
      notesByLocale: app.notesByLocale,
      language,
      assetUrl: app.assetUrl,
    });
  }
  if (binary) {
    items.push({
      id: "ollama-binary",
      type: "ollama-binary",
      name: "Ollama",
      sub: `v${binary.currentVersion} → v${binary.latestVersion}`,
    });
  }
  for (const m of models) {
    items.push({
      id: m.fullName,
      type: "ollama",
      name: m.fullName,
      sub: m.family,
      fullName: m.fullName,
    });
  }
  for (const update of forecastUpdates) {
    const current = shortVersion(update.current);
    const latest = shortVersion(update.latest);
    items.push({
      id: `forecast-dev-${update.id}`,
      type: "forecast-dev",
      name: update.displayName,
      sub: `${t(`updates.forecastDev${update.kind === "model" ? "Model" : "Runtime"}`)} · ${current} → ${latest}`,
      sourceUrl: update.sourceUrl,
    });
  }
  return items;
}

function shortVersion(value: string): string {
  return /^[a-f\d]{40}$/i.test(value) ? value.slice(0, 7) : value;
}
