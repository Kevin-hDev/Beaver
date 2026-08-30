import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useAvailableModels } from "@/hooks/use-available-models";
import { useFsEvent } from "@/hooks/use-fs-event";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect, type SelectGroup } from "./settings-select";
import { FileAccessSettings } from "./file-access-settings";
import { OllamaSettingsSection } from "./ollama-settings-section";
import { AgentImportSettings } from "@/components/agent-import/agent-import-settings";
import { SessionWorkspaceSettings } from "./session-workspace-settings";
import { notifySettingsChanged } from "@/hooks/use-setting-value";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import { ADVANCED_SETTINGS_DEFAULTS, type AdvancedSettingsState } from "./advanced-settings-state";

interface AdvancedSettingsProps {
  focusTarget?: "file-access" | null;
  onFocusTargetHandled?: () => void;
}

export function AdvancedSettings({ focusTarget, onFocusTargetHandled }: AdvancedSettingsProps) {
  const { t } = useTranslation();
  const { groups } = useAvailableModels();
  const [state, setState] = useState<AdvancedSettingsState>(ADVANCED_SETTINGS_DEFAULTS);
  const stateRef = useRef<AdvancedSettingsState>(ADVANCED_SETTINGS_DEFAULTS);

  const loadSettings = useCallback(() => {
    invoke<AdvancedSettingsState>("get_advanced_settings")
      .then((settings) => {
        stateRef.current = settings;
        setState(settings);
      })
      .catch(() => showToast(i18n.t("errors.operationFailed"), "error"));
  }, []);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useFsEvent("fs:config-changed", loadSettings);

  const save = useCallback(async (patch: Partial<AdvancedSettingsState>): Promise<boolean> => {
    const next = { ...stateRef.current, ...patch };
    stateRef.current = next;
    setState(next);
    try {
      await invoke("set_advanced_settings", { settings: next });
      notifySettingsChanged();
      return true;
    } catch {
      showToast(i18n.t("errors.saveFailed"), "error");
      loadSettings();
      return false;
    }
  }, [loadSettings]);

  const saveFromEvent = useCallback((patch: Partial<AdvancedSettingsState>): void => {
    // Generic controls do not restart Ollama, so their handled save result is intentionally ignored.
    void save(patch);
  }, [save]);

  const saveAllowedPaths = useCallback(async (paths: string[]) => {
    try {
      const normalized = await invoke<string[]>("set_allowed_paths", { paths });
      const next = { ...stateRef.current, allowed_paths: normalized };
      stateRef.current = next;
      setState(next);
      notifySettingsChanged();
    } catch {
      showToast(i18n.t("errors.saveFailed"), "error");
    }
  }, []);

  const modelGroups = useMemo((): SelectGroup[] => {
    const result: SelectGroup[] = [];
    for (const [, models] of groups) {
      if (models.length === 0) continue;
      result.push({
        label: models[0].provider_name,
        options: models.map((m) => ({
          value: `${m.provider_id}:${m.id}`,
          label: m.id,
          dimmed: false,
        })),
      });
    }
    return result;
  }, [groups]);

  const titleStyle = { fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", marginBottom: 28 } as const;
  const subStyle = { fontSize: "var(--text-base)", fontWeight: 600, color: "var(--ink)", marginTop: 28, marginBottom: 12 } as const;

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: "var(--settings-content-max-width)", width: "100%", margin: "0 auto" }}>
        <h2 style={titleStyle}>{t("settings.tabs.advanced")}</h2>

        <AgentImportSettings />

        <SettingsCard>
          <SettingsRow
            title={t("settings.advanced.trayTitle")}
            description={t("settings.advanced.trayDesc")}
          >
            <ToggleSwitch
              checked={state.show_tray}
              ariaLabel={t("settings.advanced.trayTitle")}
              onCheckedChange={(v) => saveFromEvent({ show_tray: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.advanced.defaultModelTitle")}
            description={t("settings.advanced.defaultModelDesc")}
          >
            <SettingsSelect
              groups={modelGroups}
              value={state.default_model}
              onChange={(v) => saveFromEvent({ default_model: v })}
              searchable
              searchPlaceholder={t("settings.advanced.searchModel")}
            />
          </SettingsRow>

        </SettingsCard>

        <h3 style={subStyle}>{t("settings.advanced.ollamaTitle")}</h3>

        <OllamaSettingsSection
          keepAlive={state.keep_alive}
          hardwareAccel={state.hardware_accel}
          multiModel={state.multi_model}
          showGpuStatus={state.show_gpu_status}
          onSave={save}
        />

        <FileAccessSettings
          paths={state.allowed_paths}
          focusRequested={focusTarget === "file-access"}
          onPathsChange={saveAllowedPaths}
          onFocusHandled={onFocusTargetHandled}
        />

        <SessionWorkspaceSettings
          outputsDirectory={state.session_outputs_directory}
          onOutputsDirectoryChange={(directory) => saveFromEvent({ session_outputs_directory: directory })}
        />

      </div>
    </div>
  );
}
