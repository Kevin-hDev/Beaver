import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import type { ThemeChoice } from "@/hooks/use-theme";
import { THEME_OPTIONS, type ThemeColorScheme } from "@/lib/app-themes";
import "./theme-selector.css";

interface ThemeSelectorProps {
  value: ThemeChoice;
  onChange: (theme: ThemeChoice) => void;
}

interface ThemePreviewProps {
  id: ThemeChoice;
  colorScheme: ThemeColorScheme | "system";
}

function PreviewContent() {
  return (
    <>
      <span className="ts-preview-accent" />
      <span className="ts-preview-text" />
    </>
  );
}

function ThemePreview({ id, colorScheme }: ThemePreviewProps) {
  if (colorScheme === "system") {
    return (
      <div className="ts-preview ts-preview-system" aria-hidden="true">
        <span className="ts-preview-half" data-theme="light"><PreviewContent /></span>
        <span className="ts-preview-half" data-theme="dark"><PreviewContent /></span>
      </div>
    );
  }

  return (
    <div className="ts-preview" data-theme={colorScheme} data-palette={id} aria-hidden="true">
      <PreviewContent />
    </div>
  );
}

export function ThemeSelector({ value, onChange }: ThemeSelectorProps) {
  const { t } = useTranslation();

  return (
    <div className="ts-grid">
      {THEME_OPTIONS.map((theme) => {
        const active = theme.id === value;
        return (
          <button
            type="button"
            key={theme.id}
            className={`ts-option ${active ? "is-active" : ""}`}
            onClick={() => onChange(theme.id)}
            aria-pressed={active}
          >
            <ThemePreview id={theme.id} colorScheme={theme.colorScheme} />
            <span className="ts-label">{t(theme.labelKey)}</span>
            {active && (
              <span className="ts-check" aria-hidden="true">
                <Check size="var(--icon-xs)" weight="bold" />
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
