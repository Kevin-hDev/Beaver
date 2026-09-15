import { useState } from "react";
import { useTranslation } from "react-i18next";
import { APP_SHORTCUTS, matchesAppShortcut } from "@/lib/app-shortcuts";
import { voiceShortcutValue } from "@/features/voice/voice-keyboard";

export function VoiceShortcutInput({ value, onChange }: { value: string | null; onChange: (value: string | null) => void }) {
  const { t } = useTranslation();
  const [error, setError] = useState(false);
  return <div className="vset-shortcut">
    <input className="field" value={value ?? ""} placeholder={t("voice.settings.noShortcut")} readOnly onKeyDown={(event) => {
      event.preventDefault();
      if (event.key === "Backspace" || event.key === "Delete") { setError(false); onChange(null); return; }
      const next = voiceShortcutValue(event.nativeEvent);
      if (!next) return;
      const collision = APP_SHORTCUTS.some((shortcut) => shortcut.id !== "toggleVoice" && matchesAppShortcut(event.nativeEvent, shortcut.id));
      setError(collision);
      if (!collision) onChange(next);
    }} />
    {error && <span role="alert">{t("voice.settings.shortcutConflict")}</span>}
  </div>;
}
