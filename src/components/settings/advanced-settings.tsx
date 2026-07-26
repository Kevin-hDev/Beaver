import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useAvailableModels } from "@/hooks/use-available-models";
import { useFsEvent } from "@/hooks/use-fs-event";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect, type SelectGroup } from "./settings-select";
import { PathListEditor } from "./path-list-editor";
import { OllamaSettingsSection } from "./ollama-settings-section";
import { AgentImportSettings } from "@/components/agent-import/agent-import-settings";
import { notifySettingsChanged } from "@/hooks/use-setting-value";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import "./compression-slider.css";

interface AdvancedState {
  autostart: boolean;
  start_hidden: boolean;
  show_tray: boolean;
  default_model: string;
  keep_alive: string;
  allowed_paths: string[];
  hardware_accel: string;
  multi_model: boolean;
  show_gpu_status: boolean;
  compression_enabled: boolean;
  compression_threshold: number;
  response_language: string;
  link_preview_enabled: boolean;
  ollama_setup_skipped: boolean;
}

const DEFAULTS: AdvancedState = {
  autostart: false,
  start_hidden: false,
  show_tray: true,
  default_model: "",
  keep_alive: "5m",
  allowed_paths: ["/"],
  hardware_accel: "gpu",
  multi_model: false,
  show_gpu_status: false,
  compression_enabled: true,
  compression_threshold: 85,
  response_language: "",
  link_preview_enabled: true,
  ollama_setup_skipped: false,
};

export function AdvancedSettings() {
  const { t } = useTranslation();
  const { groups } = useAvailableModels();
  const [state, setState] = useState<AdvancedState>(DEFAULTS);

  const loadSettings = useCallback(() => {
    invoke<AdvancedState>("get_advanced_settings")
      .then(setState)
      .catch(() => {});
  }, []);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useFsEvent("fs:config-changed", loadSettings);

  const save = useCallback((patch: Partial<AdvancedState>) => {
    setState((prev) => {
      const next = { ...prev, ...patch };
      invoke("set_advanced_settings", { settings: next }).catch(() => showToast(i18n.t("errors.saveFailed"), "error"));
      notifySettingsChanged();
      return next;
    });
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
          dimmed: !m.is_free && !m.is_local,
        })),
      });
    }
    return result;
  }, [groups]);

  const titleStyle = { fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", marginBottom: 28 } as const;
  const subStyle = { fontSize: "var(--text-base)", fontWeight: 600, color: "var(--ink)", marginTop: 28, marginBottom: 12 } as const;

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
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
              onCheckedChange={(v) => save({ show_tray: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.advanced.defaultModelTitle")}
            description={t("settings.advanced.defaultModelDesc")}
          >
            <SettingsSelect
              groups={modelGroups}
              value={state.default_model}
              onChange={(v) => save({ default_model: v })}
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

        <h3 style={subStyle}>{t("settings.advanced.compressionTitle")}</h3>

        <SettingsCard>
          <SettingsRow
            title={t("settings.advanced.compressionEnabledTitle")}
            description={t("settings.advanced.compressionEnabledDesc")}
          >
            <ToggleSwitch
              checked={state.compression_enabled}
              ariaLabel={t("settings.advanced.compressionEnabledTitle")}
              onCheckedChange={(v) => save({ compression_enabled: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.advanced.compressionThresholdTitle")}
            description={t("settings.advanced.compressionThresholdDesc")}
          >
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <input
                type="range"
                min={0}
                max={100}
                value={state.compression_threshold}
                disabled={!state.compression_enabled}
                onChange={(e) => save({ compression_threshold: Number(e.target.value) })}
                className="compression-slider"
                style={{ width: 120, opacity: state.compression_enabled ? 1 : 0.4, cursor: state.compression_enabled ? "pointer" : "not-allowed" }}
              />
              <span style={{ fontSize: "var(--text-sm)", color: "var(--ink-muted)", minWidth: 36, textAlign: "right" }}>
                {state.compression_threshold}%
              </span>
            </div>
          </SettingsRow>
        </SettingsCard>

        <h3 style={subStyle}>{t("settings.advanced.fileAccessTitle")}</h3>

        <SettingsCard>
          <div style={{ padding: "14px 20px" }}>
            <div style={{ fontSize: "var(--text-xs)", color: "var(--ink-muted)", marginBottom: 12 }}>
              {t("settings.advanced.fileAccessDesc")}
            </div>
            <PathListEditor
              paths={state.allowed_paths}
              onChange={(paths) => save({ allowed_paths: paths })}
            />
          </div>
        </SettingsCard>

      </div>
    </div>
  );
}
