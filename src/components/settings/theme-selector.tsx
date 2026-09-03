import { useEffect, useRef, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import type { ThemeChoice } from "@/hooks/use-theme";
import {
  RESOLVED_THEME_OPTIONS,
  type ThemeColorScheme,
} from "@/lib/app-themes";
import { applyExtensionTheme } from "@/features/extension-ui/themes/theme-application";
import { useThemeCatalog } from "@/features/extension-ui/themes/theme-context";
import type { ExtensionThemeEntry } from "@/features/extension-ui/themes/theme-catalog";
import "./theme-selector.css";

interface ThemeSelectorProps {
  value: ThemeChoice;
  onChange: (theme: ThemeChoice) => void;
}

function PreviewContent() {
  return (
    <>
      <span className="ts-preview-accent" />
      <span className="ts-preview-text" />
    </>
  );
}

function CoreThemePreview({
  id,
  colorScheme,
}: {
  id: string;
  colorScheme: ThemeColorScheme;
}) {
  return (
    <div className="ts-preview" data-theme={colorScheme} data-palette={id} aria-hidden="true">
      <PreviewContent />
    </div>
  );
}

function ExtensionThemePreview({ entry }: { entry: ExtensionThemeEntry }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (ref.current) applyExtensionTheme(ref.current, entry);
  }, [entry]);
  return <div ref={ref} className="ts-preview" aria-hidden="true"><PreviewContent /></div>;
}

function SystemThemePreview() {
  return (
    <div className="ts-preview ts-preview-system" aria-hidden="true">
      <span className="ts-preview-half" data-theme="light"><PreviewContent /></span>
      <span className="ts-preview-half" data-theme="dark"><PreviewContent /></span>
    </div>
  );
}

function ThemeOption({
  active,
  choice,
  label,
  onChange,
  preview,
  source,
}: {
  active: boolean;
  choice: ThemeChoice;
  label: string;
  onChange: (theme: ThemeChoice) => void;
  preview: ReactNode;
  source?: string;
}) {
  return (
    <button
      type="button"
      className={`ts-option ${active ? "is-active" : ""}`}
      onClick={() => onChange(choice)}
      aria-label={label}
      aria-pressed={active}
    >
      {preview}
      <span className="ts-label">{label}</span>
      {source && <span className="ts-source">{source}</span>}
      {active && (
        <span className="ts-check" aria-hidden="true">
          <Check size="var(--icon-xs)" weight="bold" />
        </span>
      )}
    </button>
  );
}

export function ThemeSelector({ value, onChange }: ThemeSelectorProps) {
  const { t } = useTranslation();
  const catalog = useThemeCatalog();

  return (
    <div className="ts-grid">
      {RESOLVED_THEME_OPTIONS.map((theme) => (
        <ThemeOption
          key={theme.id}
          active={theme.id === value}
          choice={theme.id}
          label={t(theme.labelKey)}
          onChange={onChange}
          preview={<CoreThemePreview id={theme.id} colorScheme={theme.colorScheme} />}
        />
      ))}
      {catalog.entries.map((entry) => (
        <ThemeOption
          key={entry.choice}
          active={entry.choice === value}
          choice={entry.choice}
          label={entry.label}
          source={entry.sourceName}
          onChange={onChange}
          preview={<ExtensionThemePreview entry={entry} />}
        />
      ))}
      <ThemeOption
        active={value === "system"}
        choice="system"
        label={t("settings.system")}
        onChange={onChange}
        preview={<SystemThemePreview />}
      />
    </div>
  );
}
