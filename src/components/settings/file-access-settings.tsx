import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsCard } from "./settings-card";
import { PathListEditor } from "./path-list-editor";
import "./file-access-settings.css";

export const FILE_ACCESS_HIGHLIGHT_MS = 1_800;

interface FileAccessSettingsProps {
  paths: string[];
  focusRequested: boolean;
  onPathsChange: (paths: string[]) => Promise<void>;
  onFocusHandled?: () => void;
}

export function FileAccessSettings({
  paths,
  focusRequested,
  onPathsChange,
  onFocusHandled,
}: FileAccessSettingsProps) {
  const { t } = useTranslation();
  const rootRef = useRef<HTMLElement>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!focusRequested) return;
    const root = rootRef.current;
    root?.classList.add("fas-targeted");
    const frame = requestAnimationFrame(() => {
      root?.scrollIntoView({ behavior: "smooth", block: "center" });
      root?.focus({ preventScroll: true });
    });
    const timeout = window.setTimeout(() => {
      root?.classList.remove("fas-targeted");
      onFocusHandled?.();
    }, FILE_ACCESS_HIGHLIGHT_MS);
    return () => {
      cancelAnimationFrame(frame);
      window.clearTimeout(timeout);
      root?.classList.remove("fas-targeted");
    };
  }, [focusRequested, onFocusHandled]);

  const save = async (nextPaths: string[]) => {
    if (saving) return;
    setSaving(true);
    try {
      await onPathsChange(nextPaths);
    } finally {
      setSaving(false);
    }
  };

  return (
    <section ref={rootRef} className="fas-root" tabIndex={-1} aria-labelledby="fas-title">
      <h3 id="fas-title" className="fas-heading">{t("settings.advanced.fileAccessTitle")}</h3>
      <SettingsCard className="fas-card">
        <div className="fas-content">
          <p className="fas-description">{t("settings.advanced.fileAccessDesc")}</p>
          <PathListEditor
            paths={paths}
            disabled={saving}
            onChange={(nextPaths) => void save(nextPaths)}
          />
        </div>
      </SettingsCard>
    </section>
  );
}
