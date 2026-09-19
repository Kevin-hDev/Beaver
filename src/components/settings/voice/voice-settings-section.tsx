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
  const [settingsError, setSettingsError] = useState(false);
  const [catalogError, setCatalogError] = useState(false);
  const [open, setOpen] = useState(false);
  const downloads = useModelDownloads();
  const reportFailure = useCallback(() => showToast(t("errors.operationFailed"), "error"), [t]);
  const refresh = useCallback(async () => {
    if (IS_LINUX) return;
    // Settings remain usable when the packaged model catalogue cannot be read.
    const [nextSettings, nextCatalog, nextDevices] = await Promise.allSettled([
      getVoiceSettings(), getVoiceCatalog(), listVoiceDevices(),
    ]);
    if (nextSettings.status === "fulfilled") setSettings(nextSettings.value);
    if (nextCatalog.status === "fulfilled") setCatalog(nextCatalog.value);
    setDevices(nextDevices.status === "fulfilled" ? nextDevices.value : []);
    setSettingsError(nextSettings.status === "rejected");
    setCatalogError(nextCatalog.status === "rejected");
  }, []);
  useEffect(() => {
    if (IS_LINUX) return;
    void Promise.resolve().then(refresh);
    const unlisten = listen("voice-models-changed", () => { void refresh(); });
    return () => cleanupTauriListener(unlisten);
  }, [refresh]);
  const save = async (patch: VoiceSettingsPatch) => setSettings(await updateVoiceSettings(patch));
  if (IS_LINUX) return null;
  const selected = catalog.find((item) => item.model === settings?.model);
  return <SettingsSectionFrame title={t("voice.settings.title")}>
    {!settings ? <SettingsCard><SettingsRow title={t(settingsError ? "voice.settings.settingsUnavailable" : "common.loading")}>
      {settingsError ? <button type="button" className="btn btn-sm btn-secondary" onClick={() => void refresh()}>{t("voice.settings.retry")}</button> : null}
    </SettingsRow></SettingsCard> : <>
    <SettingsCard>
      {settingsError && <SettingsRow title={t("voice.settings.settingsUnavailable")}>
        <button type="button" className="btn btn-sm btn-secondary" onClick={() => void refresh()}>{t("voice.settings.retry")}</button>
      </SettingsRow>}
      {catalogError && <SettingsRow title={t("voice.settings.catalogUnavailable")}>
        <button type="button" className="btn btn-sm btn-secondary" onClick={() => void refresh()}>{t("voice.settings.retry")}</button>
      </SettingsRow>}
      <SettingsRow title={t("voice.settings.enabled")} description={t("voice.settings.enabledDescription")}><ToggleSwitch checked={settings.enabled} ariaLabel={t("voice.settings.enabled")} onCheckedChange={(enabled) => void save({ enabled }).catch(reportFailure)} /></SettingsRow>
      <SettingsRow title={t("voice.settings.model")} description={selected ? `${t("voice.settings.languageCount", { count: selected.languages.length })} · ${selected.installed ? t("voice.settings.installed") : t("voice.settings.notInstalled")}` : undefined}><SettingsSelect value={settings.model} options={catalog.filter((item) => item.model).map((item) => ({ value: item.model!, label: voiceModelName(item.model!) }))} disabled={catalogError} onChange={(model) => void save({ model: model as VoiceSettings["model"] }).catch(reportFailure)} /></SettingsRow>
      <SettingsRow title={t("voice.settings.advanced")} description={t("voice.settings.advancedDescription")}><button type="button" className="btn btn-sm btn-secondary" onClick={() => { setOpen(true); void refresh().catch(reportFailure); }}>{t("voice.settings.open")}</button></SettingsRow>
    </SettingsCard>
    {open && <VoicePanel settings={settings} catalog={catalog} devices={devices} downloads={downloads.downloads} onClose={() => setOpen(false)} onSave={(patch) => void save(patch).catch(reportFailure)} onRefreshDevices={() => void refresh()} onInstall={(id) => void downloads.startDownload({ kind: "voice", modelId: id }).catch(reportFailure)} onResume={(id) => void downloads.resumeDownload(id).catch(reportFailure)} onRemove={(id) => void invoke("voice_dispatch", { action: { action: "uninstall", model_id: id } }).then(refresh).catch(reportFailure)} />}
    </>}
  </SettingsSectionFrame>;
}
