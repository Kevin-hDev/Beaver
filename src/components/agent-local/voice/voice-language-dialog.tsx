import { useTranslation } from "react-i18next";
import { SettingsDialog } from "@/components/ui/settings-dialog";
import type { VoiceLanguage } from "@/types/voice.generated";

export function VoiceLanguageDialog({ onStart, onClose }: { onStart: (language: VoiceLanguage | null) => void; onClose: () => void }) {
  const { t } = useTranslation();
  return (
    <SettingsDialog title={t("voice.language.title")} description={t("voice.language.description")} onClose={onClose}>
      <div className="sd-body vc-language-options">
        <button type="button" className="settings-card" onClick={() => onStart(null)}>{t("voice.language.configured")}</button>
        <button type="button" className="settings-card" onClick={() => onStart({ kind: "automatic" })}>{t("voice.language.automatic")}</button>
      </div>
    </SettingsDialog>
  );
}
