import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { IS_MAC } from "@/lib/platform";
import { showToast } from "@/lib/toast-emitter";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect, type SelectOption } from "./settings-select";
import { HardwareAccelControl } from "./hardware-accel-control";
import { VramTable } from "./vram-table";

interface OllamaSettingsProps {
  keepAlive: string;
  hardwareAccel: string;
  multiModel: boolean;
  showGpuStatus: boolean;
  onSave: (patch: Record<string, unknown>) => void;
}

export function OllamaSettingsSection({
  keepAlive, hardwareAccel, multiModel, showGpuStatus, onSave,
}: OllamaSettingsProps) {
  const { t } = useTranslation();
  const [accelChanged, setAccelChanged] = useState(false);
  const [restarting, setRestarting] = useState(false);

  const hardwareAccelOptions = useMemo((): SelectOption[] => [
    { value: "cpu", label: t("settings.advanced.hardwareAccelCpu") },
    { value: "gpu", label: t("settings.advanced.hardwareAccelGpu") },
  ], [t]);

  const keepAliveOptions = useMemo((): SelectOption[] => [
    { value: "0", label: t("settings.advanced.keepAlive.immediately") },
    { value: "2m", label: t("settings.advanced.keepAlive.2min") },
    { value: "5m", label: t("settings.advanced.keepAlive.5min") },
    { value: "10m", label: t("settings.advanced.keepAlive.10min") },
    { value: "15m", label: t("settings.advanced.keepAlive.15min") },
    { value: "30m", label: t("settings.advanced.keepAlive.30min") },
    { value: "forever", label: t("settings.advanced.keepAlive.onClose") },
  ], [t]);

  const handleRestart = async () => {
    setRestarting(true);
    try {
      const launched = await invoke<boolean>("restart_ollama_sidecar");
      const msg = launched
        ? t("settings.advanced.hardwareAccelRestarted")
        : t("settings.advanced.ollamaExternalReused");
      showToast(msg, "success");
      setAccelChanged(false);
    } catch {
      showToast(t("errors.ollamaRestartFailed"), "error");
    } finally {
      setRestarting(false);
    }
  };

  return (
    <SettingsCard>
      <SettingsRow
        title={t("settings.advanced.keepAliveTitle")}
        description={t("settings.advanced.keepAliveDesc")}
      >
        <SettingsSelect
          options={keepAliveOptions}
          value={keepAlive}
          onChange={(v) => { onSave({ keep_alive: v }); setAccelChanged(true); }}
        />
      </SettingsRow>

      {!IS_MAC && (
        <SettingsRow
          title={t("settings.advanced.hardwareAccelTitle")}
          description={t("settings.advanced.hardwareAccelDesc")}
        >
          <HardwareAccelControl
            options={hardwareAccelOptions}
            value={hardwareAccel}
            changed={accelChanged}
            restarting={restarting}
            onSelect={(v) => { onSave({ hardware_accel: v }); setAccelChanged(true); }}
            onRestart={() => void handleRestart()}
            restartLabel={t("settings.advanced.hardwareAccelRestart")}
          />
        </SettingsRow>
      )}

      <SettingsRow
        title={t("settings.advanced.multiModelTitle")}
        description={t("settings.advanced.multiModelDesc")}
      >
        <ToggleSwitch
          checked={multiModel}
          ariaLabel={t("settings.advanced.multiModelTitle")}
          onCheckedChange={(v) => { onSave({ multi_model: v }); setAccelChanged(true); }}
        />
      </SettingsRow>

      <SettingsRow
        title={t("settings.advanced.showGpuStatusTitle")}
        description={t("settings.advanced.showGpuStatusDesc")}
      >
        <ToggleSwitch
          checked={showGpuStatus}
          ariaLabel={t("settings.advanced.showGpuStatusTitle")}
          onCheckedChange={(v) => onSave({ show_gpu_status: v })}
        />
      </SettingsRow>

      {accelChanged && (
        <SettingsRow
          title={t("settings.advanced.restartRequiredTitle")}
          description={t("settings.advanced.restartRequiredDesc")}
        >
          <button
            className="ollama-btn ollama-btn-primary"
            onClick={() => void handleRestart()}
            disabled={restarting}
            style={{ whiteSpace: "nowrap" }}
          >
            {restarting ? "..." : t("settings.advanced.hardwareAccelRestart")}
          </button>
        </SettingsRow>
      )}

      <VramTable />
    </SettingsCard>
  );
}
