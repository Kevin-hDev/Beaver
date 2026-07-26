import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import type { ThemeChoice } from "@/hooks/use-theme";
import type { useSettings } from "@/hooks/use-settings";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { showToast } from "@/lib/toast-emitter";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect } from "./settings-select";
import { ThemeSelector } from "./theme-selector";
import { CodeThemePreview } from "./code-theme-preview";
import { FontSizeControl } from "./font-size-control";
import {
  CODE_THEME_OPTIONS,
  FONT_FAMILY_OPTIONS,
  LANGUAGE_OPTIONS,
  RESPONSE_LANGUAGE_OPTIONS,
} from "./general-settings-options";

interface GeneralSettingsProps {
  themeChoice: ThemeChoice;
  onThemeChange: (theme: ThemeChoice) => void;
  settings: ReturnType<typeof useSettings>;
}

interface StartupState {
  autostart: boolean;
  start_hidden: boolean;
  response_language: string;
  link_preview_enabled: boolean;
}

function normalizeStartup(state: StartupState): StartupState {
  return state.autostart ? state : { ...state, start_hidden: false };
}

export function GeneralSettings({ themeChoice, onThemeChange, settings }: GeneralSettingsProps) {
  const { t, i18n } = useTranslation();
  const [startup, setStartup] = useState<StartupState>({
    autostart: false,
    start_hidden: false,
    response_language: "",
    link_preview_enabled: true,
  });

  useEffect(() => {
    invoke<Record<string, unknown>>("get_advanced_settings")
      .then((s) => {
        const linkPreview = (s.link_preview_enabled as boolean) ?? true;
        localStorage.setItem("clgo-link-preview", String(linkPreview));
        setStartup(normalizeStartup({
          autostart: s.autostart as boolean ?? false,
          start_hidden: s.start_hidden as boolean ?? false,
          response_language: s.response_language as string ?? "",
          link_preview_enabled: linkPreview,
        }));
      })
      .catch(() => console.warn("Failed to load advanced settings"));
  }, []);

  const saveAdvanced = useCallback((patch: Partial<StartupState>) => {
    setStartup((prev) => {
      const next = normalizeStartup({ ...prev, ...patch });
      const normalizedPatch = { ...patch };
      if (!next.autostart) normalizedPatch.start_hidden = false;
      invoke("patch_advanced_settings", { patch: normalizedPatch }).catch(() => {
        setStartup(prev);
        showToast(t("errors.saveFailed"), "error");
      });
      if ("link_preview_enabled" in normalizedPatch) {
        localStorage.setItem("clgo-link-preview", String(normalizedPatch.link_preview_enabled));
      }
      return next;
    });
  }, [t]);

  const changeLang = (lang: string) => {
    void i18n.changeLanguage(lang);
    localStorage.setItem("clgo-language", lang);
  };

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
        <h2 style={{
          fontSize: "var(--text-xl)",
          fontWeight: 700,
          color: "var(--ink)",
          marginBottom: 28,
        }}>
          {t("settings.tabs.general")}
        </h2>

        <SettingsCard>
          <SettingsRow
            title={t("settings.general.themeTitle")}
            description={t("settings.general.themeDesc")}
          >
            <ThemeSelector value={themeChoice} onChange={onThemeChange} />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.fontSizeTitle")}
            description={t("settings.general.fontSizeDesc")}
          >
            <FontSizeControl value={settings.fontSize} onChange={settings.setFontSize} />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.fontFamilyTitle")}
            description={t("settings.general.fontFamilyDesc")}
          >
            <SettingsSelect
              options={FONT_FAMILY_OPTIONS}
              value={settings.fontFamilyId}
              onChange={(v) => settings.setFontFamily(v as typeof settings.fontFamilyId)}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.codeThemeTitle")}
            description={t("settings.general.codeThemeDesc")}
          >
            <SettingsSelect
              options={CODE_THEME_OPTIONS}
              value={settings.codeThemeId}
              onChange={(v) => settings.setCodeTheme(v as typeof settings.codeThemeId)}
            />
          </SettingsRow>
        </SettingsCard>

        <CodeThemePreview themeId={settings.codeThemeId} />

        <SettingsCard>
          <SettingsRow
            title={t("settings.general.languageTitle")}
            description={t("settings.general.languageDesc")}
          >
            <SettingsSelect
              options={LANGUAGE_OPTIONS}
              value={i18n.language}
              onChange={changeLang}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.responseLangTitle")}
            description={t("settings.general.responseLangDesc")}
          >
            <SettingsSelect
              options={RESPONSE_LANGUAGE_OPTIONS}
              value={startup.response_language}
              onChange={(v) => saveAdvanced({ response_language: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.sidebarExpandTitle")}
            description={t("settings.general.sidebarExpandDesc")}
          >
            <ToggleSwitch
              checked={settings.sidebarExpand}
              ariaLabel={t("settings.general.sidebarExpandTitle")}
              onCheckedChange={settings.setSidebarExpand}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.linkPreviewTitle")}
            description={t("settings.general.linkPreviewDesc")}
          >
            <ToggleSwitch
              checked={startup.link_preview_enabled}
              ariaLabel={t("settings.general.linkPreviewTitle")}
              onCheckedChange={(v) => saveAdvanced({ link_preview_enabled: v })}
            />
          </SettingsRow>
        </SettingsCard>

        <SettingsCard>
          <SettingsRow
            title={t("settings.advanced.autostartTitle")}
            description={t("settings.advanced.autostartDesc")}
          >
            <ToggleSwitch
              checked={startup.autostart}
              ariaLabel={t("settings.advanced.autostartTitle")}
              onCheckedChange={(v) => saveAdvanced({ autostart: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.advanced.startHiddenTitle")}
            description={t("settings.advanced.startHiddenDesc")}
          >
            <ToggleSwitch
              checked={startup.autostart && startup.start_hidden}
              ariaLabel={t("settings.advanced.startHiddenTitle")}
              disabled={!startup.autostart}
              onCheckedChange={(v) => saveAdvanced({ start_hidden: v })}
            />
          </SettingsRow>
        </SettingsCard>
      </div>
    </div>
  );
}
