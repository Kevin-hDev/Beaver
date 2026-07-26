import "./settings-card.css";

interface SettingsCardProps {
  children: React.ReactNode;
  className?: string;
}

export function SettingsCard({ children, className }: SettingsCardProps) {
  return (
    <div className={`settings-card ${className ?? ""}`}>
      {children}
    </div>
  );
}
