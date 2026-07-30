import type { ReactNode } from "react";
import "./settings-tabbar.css";

export interface SettingsTabbarItem<Id extends string> {
  id: Id;
  label: string;
  icon?: ReactNode;
}

interface SettingsTabbarProps<Id extends string> {
  items: ReadonlyArray<SettingsTabbarItem<Id>>;
  active: Id;
  label: string;
  onChange: (id: Id) => void;
}

export function SettingsTabbar<Id extends string>({
  items,
  active,
  label,
  onChange,
}: SettingsTabbarProps<Id>) {
  return (
    <div className="settings-tabbar" role="tablist" aria-label={label}>
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          role="tab"
          aria-selected={item.id === active}
          className={`settings-tabbar-item${item.id === active ? " active" : ""}`}
          onClick={() => onChange(item.id)}
        >
          {item.icon}
          <span className="settings-tabbar-label">{item.label}</span>
        </button>
      ))}
    </div>
  );
}
