import { useTranslation } from "react-i18next";
import { SettingsDialog } from "@/components/ui/settings-dialog";
import { SettingsCard } from "../settings-card";
import { SettingsRow } from "../settings-row";
import { SettingsSelect } from "../settings-select";
import { VoiceCaptureSettings } from "./voice-capture-settings";
import { VoiceModelList } from "./voice-model-list";
import { VoiceTrial } from "./voice-trial";
import type { ModelDownloadState } from "@/types/model-download.generated";
import type { VoiceCatalogItem, VoiceDevice, VoiceSettings, VoiceSettingsPatch } from "@/types/voice.generated";

interface Props {
  settings: VoiceSettings; catalog: VoiceCatalogItem[]; devices: VoiceDevice[]; downloads: ModelDownloadState[];
  onClose: () => void; onSave: (patch: VoiceSettingsPatch) => void; onRefreshDevices: () => void;
  onInstall: (id: string) => void; onCancel: (id: string) => void; onResume: (id: string) => void; onRemove: (id: string) => void;
}

export function VoicePanel(props: Props) {
  const { t } = useTranslation();
  const languages = props.catalog.flatMap((item) => item.languages);
  return <SettingsDialog title={t("voice.settings.title")} description={t("voice.settings.description")} onClose={props.onClose}>
    <div className="sd-body vset-body">
      <SettingsCard><VoiceModelList items={props.catalog} selected={props.settings.model} downloads={props.downloads} onSelect={(model) => props.onSave({ model })} onInstall={props.onInstall} onCancel={props.onCancel} onResume={props.onResume} onRemove={props.onRemove} /></SettingsCard>
      <SettingsCard><VoiceCaptureSettings settings={props.settings} devices={props.devices} languages={languages} onSave={props.onSave} onRefresh={props.onRefreshDevices} /></SettingsCard>
      <SettingsCard><SettingsRow title={t("voice.settings.unload")}><SettingsSelect value={props.settings.unload_delay} options={["immediately","one-minute","two-minutes","five-minutes","fifteen-minutes","on-exit"].map((value) => ({ value, label: t(`voice.settings.unloadValues.${value}`) }))} onChange={(value) => props.onSave({ unload_delay: value as VoiceSettings["unload_delay"] })} /></SettingsRow></SettingsCard>
    </div>
    <footer className="sd-foot"><VoiceTrial /></footer>
  </SettingsDialog>;
}
