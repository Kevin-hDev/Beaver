import { useTranslation } from "react-i18next";
import type { SystemPromptView } from "@/types/system-prompts";

interface SystemPromptActionsProps {
  view: SystemPromptView | null;
  isOllama: boolean;
  saving: boolean;
  onUseBeaver: () => void;
  onUseOllama: () => void;
  onEdit: () => void;
}

export function SystemPromptActions({
  view,
  isOllama,
  saving,
  onUseBeaver,
  onUseOllama,
  onEdit,
}: SystemPromptActionsProps) {
  const { t } = useTranslation();
  const selection = view?.selection;
  const hasCustomSelection = selection === "custom" || selection === "disabled";
  const canUseBeaver = isOllama
    && selection === "default"
    && view?.source !== "beaver";
  const canUseOllama = isOllama
    && view?.nativePromptAvailable
    && selection !== "default";

  return (
    <div className="spp-actions">
      {hasCustomSelection && (
        <SecondaryAction label={t("settings.systemPrompt.restore")} onClick={onUseBeaver} disabled={saving} />
      )}
      {canUseBeaver && (
        <SecondaryAction label={t("settings.systemPrompt.useBeaver")} onClick={onUseBeaver} disabled={saving} />
      )}
      {canUseOllama && (
        <SecondaryAction
          label={t("settings.systemPrompt.useOllama")}
          onClick={onUseOllama}
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
