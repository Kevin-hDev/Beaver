import { useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";

interface SystemPromptEditorCardProps {
  initialContent: string;
  saving: boolean;
  error: boolean;
  onCancel: () => void;
  onSave: (content: string) => void;
}

export function SystemPromptEditorCard({
  initialContent,
  saving,
  error,
  onCancel,
  onSave,
}: SystemPromptEditorCardProps) {
  const { t } = useTranslation();
  const [content, setContent] = useState(initialContent);

  return (
    <SettingsCard className="spp-card spp-editor-card">
      <div className="spp-card-header">
        <span className="spp-card-title">{t("settings.systemPrompt.instructions")}</span>
        <div className="spp-actions">
          <button className="btn btn-sm btn-secondary" onClick={onCancel} disabled={saving}>
            {t("settings.systemPrompt.cancel")}
          </button>
          <button
            className="btn btn-sm btn-primary"
            onClick={() => onSave(content)}
            disabled={saving}
          >
            {saving ? "…" : t("settings.systemPrompt.save")}
          </button>
        </div>
      </div>
      {error && <div className="spp-error" role="alert">{t("errors.operationFailed")}</div>}
      <textarea
        className="spp-textarea"
        value={content}
        onChange={(event) => setContent(event.target.value)}
        aria-label={t("settings.systemPrompt.editorLabel")}
        placeholder={t("settings.systemPrompt.placeholder")}
      />
    </SettingsCard>
  );
}
