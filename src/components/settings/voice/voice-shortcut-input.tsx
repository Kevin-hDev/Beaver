import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { APP_SHORTCUTS, matchesAppShortcut } from "@/lib/app-shortcuts";
import { ALT_LABEL, IS_MAC } from "@/lib/platform";
import { voiceShortcutValue } from "@/features/voice/voice-keyboard";

export function formatVoiceShortcut(value: string): string {
  return value.split("+").map((part) => {
    if (part === "Meta") return IS_MAC ? "⌘" : "Meta";
    if (part === "Control") return "Ctrl";
    if (part === "Alt") return ALT_LABEL;
    if (part.startsWith("Key")) return part.slice(3);
    if (part.startsWith("Digit")) return part.slice(5);
    return part;
  }).join(" + ");
}

export function VoiceShortcutInput({ value, onChange }: { value: string | null; onChange: (value: string | null) => void }) {
  const { t } = useTranslation();
  const [capturing, setCapturing] = useState(false);
  const [error, setError] = useState(false);

  useEffect(() => {
    if (!capturing) return;
    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (event.repeat) return;
      if (event.key === "Escape") { setCapturing(false); setError(false); return; }
      if (event.key === "Backspace" || event.key === "Delete") {
        setCapturing(false);
        setError(false);
        onChange(null);
        return;
      }
      const next = voiceShortcutValue(event);
      if (!next) return;
      const collision = APP_SHORTCUTS.some((shortcut) => matchesAppShortcut(event, shortcut.id));
      setError(collision);
      if (!collision) {
        setCapturing(false);
        onChange(next);
      }
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [capturing, onChange]);

  return <div className="vset-shortcut">
    <span className="vset-shortcut-value" aria-live="polite">
      {capturing ? t("voice.settings.pressShortcut") : value ? formatVoiceShortcut(value) : t("voice.settings.noShortcut")}
    </span>
    <button type="button" className="btn btn-sm btn-secondary" onClick={() => { setError(false); setCapturing((current) => !current); }}>
      {capturing ? t("voice.settings.cancelShortcut") : value ? t("voice.settings.modifyShortcut") : t("voice.settings.createShortcut")}
    </button>
    {error && <span role="alert">{t("voice.settings.shortcutConflict")}</span>}
  </div>;
}
