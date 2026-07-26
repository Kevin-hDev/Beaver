interface SettingsRowProps {
  title: string;
  description?: string;
  children: React.ReactNode;
  className?: string;
}

export function SettingsRow({ title, description, children, className }: SettingsRowProps) {
  return (
    <div className={`settings-row ${className ?? ""}`}>
      <div className="settings-row-info">
        <div className="settings-row-title">{title}</div>
        {description && <div className="settings-row-desc">{description}</div>}
      </div>
      <div className="settings-row-control">
        {children}
      </div>
    </div>
  );
}
