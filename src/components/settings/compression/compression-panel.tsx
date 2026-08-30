import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { DialogPortal } from "@/components/ui/dialog-portal";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import { CompressionProfileBar } from "./compression-profile-bar";
import { CompressionProfileEditor } from "./compression-profile-editor";
import "./compression-panel.css";
import "./compression-sections.css";

interface CompressionPanelProps {
  controller: CompressionProfilesController;
  currentWindow: number;
  onClose: () => void;
}

export function CompressionPanel({ controller, currentWindow, onClose }: CompressionPanelProps) {
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

          {controller.view && (() => {
            const active = controller.view.profiles.find(
              (profile) => profile.id === controller.view?.global_profile_id,
            );
            return active ? (
              <CompressionProfileEditor
                key={active.id}
                profile={active}
                currentWindow={currentWindow}
                controller={controller}
              />
            ) : null;
          })()}
        </section>
      </div>
    </DialogPortal>
  );
}
