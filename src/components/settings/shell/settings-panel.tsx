import type { ReactNode } from "react";
import "./settings-panel.css";

interface SettingsPanelProps {
  title?: string;
  action?: ReactNode;
  /** Élargit la colonne pour les pages dont les lignes portent icône et
      commandes, à l'étroit dans la largeur standard. */
  wide?: boolean;
  children: ReactNode;
}

export function SettingsPanel({ title, action, wide = false, children }: SettingsPanelProps) {
  const hasHeader = title !== undefined || action !== undefined;
  return (
    <div className="settings-panel">
      <div className={`settings-panel-inner${wide ? " settings-panel-inner-wide" : ""}`}>
        {hasHeader && (
          <header className="settings-panel-header">
            {title !== undefined && <h2 className="settings-panel-title">{title}</h2>}
            {action}
          </header>
        )}
        {children}
      </div>
    </div>
  );
}
