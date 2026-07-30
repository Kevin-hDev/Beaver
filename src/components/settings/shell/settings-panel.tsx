import type { ReactNode } from "react";
import "./settings-panel.css";

interface SettingsPanelProps {
  title?: string;
  action?: ReactNode;
  children: ReactNode;
}

export function SettingsPanel({ title, action, children }: SettingsPanelProps) {
  const hasHeader = title !== undefined || action !== undefined;
  return (
    <div className="settings-panel">
      <div className="settings-panel-inner">
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
