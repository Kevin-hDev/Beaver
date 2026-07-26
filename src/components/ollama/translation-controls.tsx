import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { CustomSelect } from "@/components/ui/custom-select";
import "./ollama.css";

const LANGUAGES: { code: string; label: string }[] = [
  { code: "en", label: "English" },
  { code: "fr", label: "Français" },
  { code: "es", label: "Español" },
  { code: "de", label: "Deutsch" },
  { code: "it", label: "Italiano" },
  { code: "zh", label: "中文" },
  { code: "ja", label: "日本語" },
];

interface TranslationControlsProps {
  modelName: string;
  originalText: string;
  currentLang: string | null;
  onChange: (lang: string | null, translatedText: string | null) => void;
}

export function TranslationControls({
  modelName, originalText, currentLang, onChange,
}: TranslationControlsProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const translate = async (lang: string) => {
    setLoading(true);
    setError(null);
    try {
      const translated = await invoke<string>("translate_description", {
        modelName,
        text: originalText,
        targetLang: lang,
      });
      onChange(lang, translated);
    } catch {
      setError(t("errors.operationFailed"));
      onChange(null, null);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{
      display: "flex", alignItems: "center", gap: 8,
      padding: "var(--space-sm) var(--space-md)",
      borderBottom: "1px solid var(--edge)",
      fontSize: "var(--text-xs)",
      color: "var(--ink-faint)",
    }}>
      <span style={{ textTransform: "uppercase", letterSpacing: "0.05em" }}>
        {t("ollama.description")}
      </span>

      <div style={{ flex: 1 }} />

      {loading && (
        <span style={{ color: "var(--ink-faint)", fontStyle: "italic" }}>
          {t("ollama.translating")}
        </span>
      )}

      {!loading && error && (
        <span
          style={{ color: "var(--signal-error)", maxWidth: 280, overflow: "hidden", textOverflow: "ellipsis" }}
          title={error}
        >
          {error}
        </span>
      )}

      {!loading && currentLang && (
        <button className="ollama-btn" onClick={() => onChange(null, null)}>
          {t("ollama.original")}
        </button>
      )}

      <div style={{ width: 120 }}>
        <CustomSelect
          value={currentLang ?? ""}
          onChange={(lang) => { if (lang) void translate(lang); }}
          disabled={loading}
          placeholder={t("ollama.translate")}
          options={LANGUAGES.map((l) => ({ value: l.code, label: l.label }))}
        />
      </div>
    </div>
  );
}
