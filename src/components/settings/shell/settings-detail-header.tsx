import type { ReactNode } from "react";
import { SettingsBackButton } from "./settings-back-button";
import "./settings-detail-header.css";

interface SettingsDetailHeaderProps {
  title: string;
  subtitle?: string;
  icon?: ReactNode;
  actions?: ReactNode;
  onBack: () => void;
}

export function SettingsDetailHeader({
  title,
  subtitle,
  icon,
  actions,
  onBack,
}: SettingsDetailHeaderProps) {
  return (
    <header className="settings-detail-header">
      <SettingsBackButton onClick={onBack} />
      {icon && <span className="settings-detail-icon">{icon}</span>}
      <div className="settings-detail-title">
        <h2>{title}</h2>
        {subtitle && <p>{subtitle}</p>}
      </div>
      {actions && <div className="settings-detail-actions">{actions}</div>}
    </header>
  );
}
