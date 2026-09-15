import { useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsDialog } from "@/components/ui/settings-dialog";
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
  const [interactionActive, setInteractionActive] = useState(false);

  return (
    <SettingsDialog
      title={t("settings.advanced.compressionPanelTitle")}
      description={t("settings.advanced.compressionPanelDesc")}
      closeLabel={t("settings.advanced.compressionClose")}
      onClose={onClose}
      childDialogActive={interactionActive}
    >
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
            limits={controller.view.limits}
            automaticEnabled={controller.view.automatic_enabled}
          />
        ) : null;
      })()}
    </SettingsDialog>
  );
}
