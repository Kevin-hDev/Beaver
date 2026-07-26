import type { ReactNode } from "react";
import { SettingsCard } from "@/components/settings/settings-card";
import { cn } from "@/lib/utils";
import "./model-editor-shell.css";

interface ModelEditorShellProps {
  title: string;
  cancelLabel: string;
  saveLabel: string;
  saving: boolean;
  fillAvailableSpace?: boolean;
  error?: string | null;
  onCancel: () => void;
  onSave: () => void;
  children: ReactNode;
}

export function ModelEditorShell({
  title,
  cancelLabel,
  saveLabel,
  saving,
  fillAvailableSpace = false,
  error,
  onCancel,
  onSave,
  children,
}: ModelEditorShellProps) {
  return (
    <div className={cn("mes-page", fillAvailableSpace && "mes-page-fill")}>
      <div className="mes-inner">
        <div className="mes-header">
          <h2 className="mes-title">{title}</h2>
          <div className="mes-actions">
            <button className="ollama-btn" onClick={onCancel} disabled={saving}>
              {cancelLabel}
            </button>
            <button
              className="ollama-btn ollama-btn-primary"
              onClick={onSave}
              disabled={saving}
            >
              {saving ? "..." : saveLabel}
            </button>
          </div>
        </div>

        <SettingsCard className="mes-card">
          {error && (
            <div className="mes-error" role="alert">
              {error}
            </div>
          )}
          {children}
        </SettingsCard>
      </div>
    </div>
  );
}
