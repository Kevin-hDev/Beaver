import { useEffect, useLayoutEffect, useCallback, type RefObject, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { AppUpdate, DismissedUpdate, OllamaModelUpdate, OllamaBinaryUpdate, PullingState } from "@/hooks/use-update-checker";
import type { ForecastDevUpdate } from "@/hooks/use-forecast-dev-updates";
import { createPortal } from "react-dom";
import { useFloatingMenuPosition, floatingMenuPortalRoot } from "@/hooks/use-floating-menu-position";
import { buildUpdateItems } from "./update-notification-items";
import { BubbleItem } from "./bubble-item";
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
  onDismissUpdate: (update: DismissedUpdate) => void;
  onCancelApp: () => void;
  onCancelOllamaBinary: () => void;
  onCancelModel: () => void;
  appCancelling: boolean;
  ollamaBinaryCancelling: boolean;
  modelCancelling: boolean;
  anchorRef: RefObject<HTMLElement | null>;
  children?: ReactNode;
}

export function UpdateNotifications({
  isOpen, onClose,
  appUpdate, ollamaBinaryUpdate, ollamaUpdates, forecastDevUpdates,
  pulling, ollamaBinaryUpdating, ollamaBinaryPercent,
  appDownloading, appPercent,
  onPullModel, onDownloadApp, onUpdateOllamaBinary, onDismissUpdate,
  onCancelApp, onCancelOllamaBinary, onCancelModel,
  appCancelling, ollamaBinaryCancelling, modelCancelling,
  anchorRef, children,
}: UpdateNotificationsProps) {
  const { t, i18n } = useTranslation();
  const { floatingRef, floatingStyle } = useFloatingMenuPosition(
    isOpen, "left", undefined, "below", false, undefined, undefined, anchorRef,
  );
  const items = buildUpdateItems(
    t, i18n.language, appUpdate, ollamaBinaryUpdate, ollamaUpdates, forecastDevUpdates,
  );
  const handleClose = useCallback(() => {
    // Closing only hides the surface; return focus immediately without delaying navigation.
    onClose();
    anchorRef.current?.focus();
  }, [onClose, anchorRef]);

  useLayoutEffect(() => {
    if (isOpen) floatingRef.current?.querySelector<HTMLElement>("button:not([disabled])")?.focus();
  }, [isOpen, floatingRef]);

  useEffect(() => {
    if (!isOpen) return;
    const onEscape = (e: KeyboardEvent) => {
      if (e.code === "Escape") handleClose();
    };
    window.addEventListener("keydown", onEscape);
    return () => window.removeEventListener("keydown", onEscape);
  }, [isOpen, handleClose]);

  if (!isOpen) return null;

  return createPortal(
    <>
      <div className="update-overlay" role="presentation" onClick={handleClose} onKeyDown={() => {}} />
      {/* The shared surface CSS owns its content and viewport width cap. */}
      <div ref={floatingRef} className="update-list" style={{ ...floatingStyle, maxWidth: undefined }} role="region" aria-label={t("extensionInstalls.title")}>
        {children}
        {items.map((item, i) => (
          <BubbleItem
            key={item.id}
            item={item}
            index={i}
            closing={false}
            totalCount={items.length}
            pulling={pulling}
            ollamaBinaryUpdating={ollamaBinaryUpdating}
            ollamaBinaryPercent={ollamaBinaryPercent}
            appDownloading={appDownloading}
            appPercent={appPercent}
            onPullModel={onPullModel}
            onDownloadApp={onDownloadApp}
            onUpdateOllamaBinary={onUpdateOllamaBinary}
            onDismissUpdate={onDismissUpdate}
            onCancelApp={onCancelApp}
            onCancelOllamaBinary={onCancelOllamaBinary}
            onCancelModel={onCancelModel}
            appCancelling={appCancelling}
            ollamaBinaryCancelling={ollamaBinaryCancelling}
            modelCancelling={modelCancelling}
            t={t}
          />
        ))}
      </div>
    </>, floatingMenuPortalRoot()
  );
}
