import { useTranslation } from "react-i18next";
import { SettingsDialog } from "@/components/ui/settings-dialog";

export function VoiceFirstUseDialog({ onAccept, onClose }: { onAccept: () => void; onClose: () => void }) {
  const { t } = useTranslation();
  return (
    <SettingsDialog title={t("voice.firstUse.title")} description={t("voice.firstUse.description")} onClose={onClose}>
      <div className="sd-body"><p>{t("voice.firstUse.local")}</p></div>
      <footer className="sd-foot vc-dialog-actions">
        <button type="button" className="btn btn-sm btn-secondary" onClick={onClose}>{t("voice.firstUse.later")}</button>
        <button type="button" className="btn btn-sm btn-primary" onClick={onAccept}>{t("voice.firstUse.continue")}</button>
      </footer>
    </SettingsDialog>
  );
}
