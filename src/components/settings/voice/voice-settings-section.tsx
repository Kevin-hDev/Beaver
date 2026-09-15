import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { showToast } from "@/lib/toast-emitter";
import { IS_LINUX } from "@/lib/platform";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsSectionFrame } from "@/components/ui/settings-section-frame";
import { SettingsCard } from "../settings-card";
import { SettingsRow } from "../settings-row";
import { SettingsSelect } from "../settings-select";
import { useModelDownloads } from "@/hooks/use-model-downloads";
import { getVoiceCatalog, getVoiceSettings, listVoiceDevices, updateVoiceSettings } from "@/features/voice/voice-client";
import type { VoiceCatalogItem, VoiceDevice, VoiceSettings, VoiceSettingsPatch } from "@/types/voice.generated";
import { VoicePanel } from "./voice-panel";
import { voiceModelName } from "./voice-model-list";
import "./voice-settings.css";

export function VoiceSettingsSection() {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<VoiceSettings | null>(null);
  const [catalog, setCatalog] = useState<VoiceCatalogItem[]>([]);
  const [devices, setDevices] = useState<VoiceDevice[]>([]);
  const [open, setOpen] = useState(false);
  const downloads = useModelDownloads();
  const refresh = useCallback(async () => {
    if (IS_LINUX) return;
    const [nextSettings, nextCatalog, nextDevices] = await Promise.all([getVoiceSettings(), getVoiceCatalog(), listVoiceDevices().catch(() => [])]);
    setSettings(nextSettings); setCatalog(nextCatalog); setDevices(nextDevices);
  }, []);
  useEffect(() => {
    if (IS_LINUX) return;
    void Promise.resolve().then(refresh);
    const unlisten = listen("voice-models-changed", () => { void refresh(); });
    return () => cleanupTauriListener(unlisten);
  }, [refresh]);
  const save = async (patch: VoiceSettingsPatch) => setSettings(await updateVoiceSettings(patch));
  const reportFailure = () => showToast(t("errors.operationFailed"), "error");
  if (IS_LINUX || !settings) return null;
  const selected = catalog.find((item) => item.model === settings.model);
  return <SettingsSectionFrame title={t("voice.settings.title")}>
    <SettingsCard>
      <SettingsRow title={t("voice.settings.enabled")} description={t("voice.settings.enabledDescription")}><ToggleSwitch checked={settings.enabled} ariaLabel={t("voice.settings.enabled")} onCheckedChange={(enabled) => void save({ enabled }).catch(reportFailure)} /></SettingsRow>
      <SettingsRow title={t("voice.settings.model")} description={selected ? `${t("voice.settings.languageCount", { count: selected.languages.length })} · ${selected.installed ? t("voice.settings.installed") : t("voice.settings.notInstalled")}` : undefined}><SettingsSelect value={settings.model} options={catalog.filter((item) => item.model).map((item) => ({ value: item.model!, label: voiceModelName(item.model!) }))} onChange={(model) => void save({ model: model as VoiceSettings["model"] }).catch(reportFailure)} /></SettingsRow>
      <SettingsRow title={t("voice.settings.advanced")} description={t("voice.settings.advancedDescription")}><button type="button" className="btn btn-sm btn-secondary" onClick={() => { setOpen(true); void refresh().catch(reportFailure); }}>{t("voice.settings.open")}</button></SettingsRow>
    </SettingsCard>
    {open && <VoicePanel settings={settings} catalog={catalog} devices={devices} downloads={downloads.downloads} onClose={() => setOpen(false)} onSave={(patch) => void save(patch).catch(reportFailure)} onRefreshDevices={() => void refresh().catch(reportFailure)} onInstall={(id) => void downloads.startDownload({ kind: "voice", modelId: id }).catch(reportFailure)} onResume={(id) => void downloads.resumeDownload(id).catch(reportFailure)} onRemove={(id) => void invoke("voice_dispatch", { action: { action: "uninstall", model_id: id } }).then(refresh).catch(reportFailure)} />}
  </SettingsSectionFrame>;
}
