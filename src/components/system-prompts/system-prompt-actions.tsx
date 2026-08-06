import { useTranslation } from "react-i18next";
import type { SystemPromptView } from "@/types/system-prompts";

interface SystemPromptActionsProps {
  view: SystemPromptView | null;
  isOllama: boolean;
  nativePromptAvailable: boolean;
  saving: boolean;
  onUseBeaver: () => void;
  onRestoreDefault: () => void;
  onEdit: () => void;
}

export function SystemPromptActions({
  view,
  isOllama,
  nativePromptAvailable,
  saving,
  onUseBeaver,
  onRestoreDefault,
  onEdit,
}: SystemPromptActionsProps) {
  const { t } = useTranslation();
  const selection = view?.selection;
  const hasCustomSelection = selection === "custom" || selection === "disabled";
  const canUseBeaver = isOllama
    && selection === "default"
    && view?.source !== "beaver";
  const canRestoreDefault = isOllama
    && (selection === "beaver" || hasCustomSelection);

  return (
    <div className="spp-actions">
      {!isOllama && hasCustomSelection && (
        <SecondaryAction label={t("settings.systemPrompt.restore")} onClick={onUseBeaver} disabled={saving} />
      )}
      {isOllama && hasCustomSelection && (
        <SecondaryAction label={t("settings.systemPrompt.restore")} onClick={onUseBeaver} disabled={saving} />
      )}
      {canUseBeaver && (
        <SecondaryAction label={t("settings.systemPrompt.useBeaver")} onClick={onUseBeaver} disabled={saving} />
      )}
      {canRestoreDefault && (
        <SecondaryAction
          label={t(nativePromptAvailable
            ? "settings.systemPrompt.restoreOllama"
            : "settings.systemPrompt.restoreDefault")}
          onClick={onRestoreDefault}
          disabled={saving}
        />
      )}
      <button className="btn btn-sm btn-primary" onClick={onEdit} disabled={!view || saving}>
        {t("settings.systemPrompt.edit")}
      </button>
    </div>
  );
}

function SecondaryAction({
  label,
  onClick,
  disabled,
}: {
  label: string;
  onClick: () => void;
  disabled: boolean;
}) {
  return (
    <button className="btn btn-sm btn-secondary" onClick={onClick} disabled={disabled}>
      {label}
    </button>
  );
}
