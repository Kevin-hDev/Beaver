import { invoke } from "@tauri-apps/api/core";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { useTranslation } from "react-i18next";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import "./session-workspace-settings.css";

interface SessionWorkspaceSettingsProps {
  outputsDirectory: string;
  onOutputsDirectoryChange: (directory: string) => void;
}

export function SessionWorkspaceSettings({
  outputsDirectory,
  onOutputsDirectoryChange,
}: SessionWorkspaceSettingsProps) {
  const { t } = useTranslation();

  const chooseOutputsDirectory = async () => {
    const selected = await openFileDialog({ directory: true });
    if (typeof selected === "string") {
      onOutputsDirectoryChange(selected);
    }
  };

  const openDataDirectory = async () => {
    try {
      await invoke("open_app_data_folder");
    } catch {
      showToast(i18n.t("errors.operationFailed"), "error");
    }
  };

  return (
    <>
      <h3 className="sws-title">{t("settings.advanced.sessionFilesTitle")}</h3>
      <SettingsCard>
        <SettingsRow
          title={t("settings.advanced.outputsDirectoryTitle")}
          description={t("settings.advanced.outputsDirectoryDesc")}
        >
          <div className="sws-output-control">
            <span className="sws-path" title={outputsDirectory}>
              {outputsDirectory || t("settings.advanced.outputsDirectoryDefault")}
            </span>
            {outputsDirectory && (
              <button
                type="button"
                className="btn btn-sm btn-ghost"
                onClick={() => onOutputsDirectoryChange("")}
              >
                {t("settings.advanced.outputsDirectoryReset")}
              </button>
            )}
            <button
              type="button"
              className="btn btn-sm btn-secondary"
              onClick={() => void chooseOutputsDirectory()}
            >
              {t("settings.advanced.outputsDirectoryChoose")}
            </button>
          </div>
        </SettingsRow>
        <SettingsRow
          title={t("settings.advanced.dataFolderTitle")}
          description={t("settings.advanced.dataFolderDesc")}
        >
          <button
            type="button"
            className="btn btn-sm btn-secondary"
            onClick={() => void openDataDirectory()}
          >
            {t("settings.advanced.dataFolderOpen")}
          </button>
        </SettingsRow>
      </SettingsCard>
    </>
  );
}
