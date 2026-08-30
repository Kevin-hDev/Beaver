import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { DialogPortal } from "@/components/ui/dialog-portal";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import { CompressionProfileBar } from "./compression-profile-bar";
import "./compression-panel.css";

interface CompressionPanelProps {
  controller: CompressionProfilesController;
  onClose: () => void;
}

export function CompressionPanel({ controller, onClose }: CompressionPanelProps) {
  const { t } = useTranslation();
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);
  const [interactionActive, setInteractionActive] = useState(false);

  useEffect(() => {
    closeRef.current?.focus();
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !interactionActive) onClose();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [interactionActive, onClose]);

  return (
    <DialogPortal>
      <div className="cpa-overlay">
        <button
          type="button"
          className="cpa-backdrop-dismiss"
          aria-label={t("settings.advanced.compressionClose")}
          onClick={onClose}
        />
        <section className="cpa-dialog relief" role="dialog" aria-modal="true" aria-labelledby={titleId}>
          <header className="cpa-head">
            <div>
              <h2 id={titleId}>{t("settings.advanced.compressionPanelTitle")}</h2>
              <p>{t("settings.advanced.compressionPanelDesc")}</p>
            </div>
            <button
              ref={closeRef}
              type="button"
              className="icon-btn icon-btn-secondary"
              aria-label={t("settings.advanced.compressionClose")}
              onClick={onClose}
            >
              <X size="var(--icon-sm)" />
            </button>
          </header>

          <CompressionProfileBar
            controller={controller}
            onInteractionChange={setInteractionActive}
          />

          <div className="cpa-body">
            <div className="cpa-placeholder relief">
              {t("settings.advanced.compressionPanelPlaceholder")}
            </div>
          </div>

          <footer className="cpa-foot" aria-hidden="true" />
        </section>
      </div>
    </DialogPortal>
  );
}
