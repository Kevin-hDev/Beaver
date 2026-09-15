import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { SettingsRow } from "../settings-row";
import { SettingsSelect } from "../settings-select";
import type { VoiceDevice, VoiceLanguageMode, VoiceSettings, VoiceSettingsPatch } from "@/types/voice.generated";
import { voiceLanguageOptions } from "@/features/voice/voice-language-options";
import { VoiceShortcutInput } from "./voice-shortcut-input";

export function VoiceCaptureSettings({ settings, devices, languages, languageMode, onSave, onRefresh }: {
  settings: VoiceSettings; devices: VoiceDevice[]; languages: string[]; languageMode: VoiceLanguageMode;
  onSave: (patch: VoiceSettingsPatch) => void; onRefresh: () => void;
}) {
  const { t, i18n } = useTranslation();
  const languageOptions = useMemo(() => [
    { value: "follow-interface", label: t("voice.settings.followInterface") },
    ...(languageMode === "explicit-only" ? [] : [{ value: "automatic", label: t("voice.language.automatic") }]),
    ...voiceLanguageOptions(languages, i18n.language),
  ], [i18n.language, languageMode, languages, t]);
  const languageValue = settings.language.kind === "automatic" && languageMode === "explicit-only"
    ? "follow-interface"
    : settings.language.kind === "language" ? settings.language.value : settings.language.kind;
  return <>
    <SettingsRow title={t("voice.settings.device")}>
      {devices.length ? <SettingsSelect value={settings.input_device.kind === "device" ? settings.input_device.value : "default"} options={[{ value: "default", label: t("voice.settings.systemDevice") }, ...devices.map((device) => ({ value: device.id, label: device.name }))]} onChange={(value) => onSave({ input_device: value === "default" ? { kind: "system-default" } : { kind: "device", value } })} />
        : <button type="button" className="btn btn-sm btn-secondary" onClick={onRefresh}>{t("voice.settings.refreshDevice")}</button>}
    </SettingsRow>
    <SettingsRow title={t("voice.settings.silence")}><SettingsSelect value={settings.silence_timeout} options={[3,5,10,20,30].map((value) => ({ value: `${value === 3 ? "three" : value === 5 ? "five" : value === 10 ? "ten" : value === 20 ? "twenty" : "thirty"}-seconds`, label: t("voice.settings.seconds", { count: value }) })).concat([{ value: "never", label: t("voice.settings.never") }])} onChange={(value) => onSave({ silence_timeout: value as VoiceSettings["silence_timeout"] })} /></SettingsRow>
    <SettingsRow title={t("voice.settings.duration")}><SettingsSelect value={settings.max_duration} options={[2,5,10,20,30].map((value) => ({ value: `${value}-minutes`, label: t("voice.settings.minutes", { count: value }) }))} onChange={(value) => onSave({ max_duration: value as VoiceSettings["max_duration"] })} /></SettingsRow>
    <SettingsRow title={t("voice.settings.language")}><SettingsSelect value={languageValue} options={languageOptions} searchable onChange={(value) => onSave({ language: value === "follow-interface" ? { kind: "follow-interface" } : value === "automatic" ? { kind: "automatic" } : { kind: "language", value } })} /></SettingsRow>
    <SettingsRow title={t("voice.settings.shortcut")}><VoiceShortcutInput value={settings.shortcut} onChange={(shortcut) => onSave({ shortcut })} /></SettingsRow>
  </>;
}
