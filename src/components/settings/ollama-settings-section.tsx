import { useState, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { IS_MAC } from "@/lib/platform";
import { showToast } from "@/lib/toast-emitter";
import { ollamaErrorKey } from "@/lib/ollama-runtime-error";
import { useOllamaRuntimeStatus } from "@/hooks/use-ollama-runtime-status";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect, type SelectOption } from "./settings-select";
import { HardwareAccelControl } from "./hardware-accel-control";
import { VramTable } from "./vram-table";
import { classifyOllamaRestartOutcome } from "./ollama-restart-outcome";
import type { OllamaStartOutcome } from "@/types/ollama-runtime";

interface OllamaSettingsProps {
  keepAlive: string;
  hardwareAccel: string;
  multiModel: boolean;
  showGpuStatus: boolean;
  onSave: (patch: Record<string, unknown>) => Promise<boolean>;
}

export function OllamaSettingsSection({
  keepAlive, hardwareAccel, multiModel, showGpuStatus, onSave,
}: OllamaSettingsProps) {
  const { t } = useTranslation();
  const [accelChanged, setAccelChanged] = useState(false);
  const [restarting, setRestarting] = useState(false);
  const pendingRestartSave = useRef<Promise<boolean>>(Promise.resolve(true));
  const runtime = useOllamaRuntimeStatus();

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
      if (!await pendingRestartSave.current) return;
      const outcome = await invoke<OllamaStartOutcome>("restart_ollama_sidecar");
      const presentation = classifyOllamaRestartOutcome(outcome);
      await runtime.refresh();
      if (presentation.kind === "failed") {
        showToast(t(ollamaErrorKey(presentation.code)), "error");
        return;
      }
      const message = presentation.kind === "external"
        ? t("settings.advanced.ollamaExternalReused")
        : t("settings.advanced.hardwareAccelRestarted");
      showToast(message, "success");
      if (presentation.kind === "owned") setAccelChanged(false);
    } catch (caught) {
      showToast(t(ollamaErrorKey(caught)), "error");
    } finally {
      setRestarting(false);
    }
  };

  const saveRestartSetting = (patch: Record<string, unknown>) => {
    pendingRestartSave.current = onSave(patch);
    setAccelChanged(true);
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
          onChange={(v) => saveRestartSetting({ keep_alive: v })}
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
            onSelect={(v) => saveRestartSetting({ hardware_accel: v })}
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
          onCheckedChange={(v) => saveRestartSetting({ multi_model: v })}
        />
      </SettingsRow>

      <SettingsRow
        title={t("settings.advanced.showGpuStatusTitle")}
        description={t("settings.advanced.showGpuStatusDesc")}
      >
        <ToggleSwitch
          checked={showGpuStatus}
          ariaLabel={t("settings.advanced.showGpuStatusTitle")}
          onCheckedChange={(v) => { void onSave({ show_gpu_status: v }); }}
        />
      </SettingsRow>

      {accelChanged && (
        <SettingsRow
          title={t("settings.advanced.restartRequiredTitle")}
          description={t("settings.advanced.restartRequiredDesc")}
        >
          <button
            className="btn btn-sm btn-primary"
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
