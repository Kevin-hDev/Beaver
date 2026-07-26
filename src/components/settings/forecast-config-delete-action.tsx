import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { showToast } from "@/lib/toast-emitter";
import { ConfirmButton } from "./confirm-button";

interface ForecastConfigDeleteActionProps {
  modelId: string;
  disabled: boolean;
  onDeleted: () => void;
}

export function ForecastConfigDeleteAction({
  modelId,
  disabled,
  onDeleted,
}: ForecastConfigDeleteActionProps) {
  const { t } = useTranslation();
  const [deleting, setDeleting] = useState(false);

  const handleDelete = useCallback(async () => {
    setDeleting(true);
    try {
      await invoke("uninstall_forecast_model", { name: modelId });
      onDeleted();
    } catch {
      showToast(t("errors.operationFailed"), "error");
    } finally {
      setDeleting(false);
    }
  }, [modelId, onDeleted, t]);

  return (
    <ConfirmButton
      className="ollama-btn fs-delete-model"
      label={deleting ? t("forecast.models.uninstalling") : t("forecast.models.uninstall")}
      confirmLabel={t("forecast.models.confirmUninstall")}
      onConfirm={() => void handleDelete()}
      disabled={disabled || deleting}
    />
  );
}
