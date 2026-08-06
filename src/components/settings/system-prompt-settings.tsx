import { useTranslation } from "react-i18next";
import { SystemPromptSettingsPanel } from "@/components/system-prompts/system-prompt-settings-panel";
import "./system-prompt-settings.css";

export function SystemPromptSettings() {
  const { t } = useTranslation();
  return (
    <div className="sps-page">
      <div className="sps-inner">
        <h2 className="sps-title">{t("settings.systemPrompt.title")}</h2>
        <SystemPromptSettingsPanel
          target={{ scope: "global" }}
          warningKind="global"
          initialMode="agentic"
          initialTier="detailed"
        />
      </div>
    </div>
  );
}
