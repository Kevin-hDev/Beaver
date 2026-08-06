import type { ReactNode } from "react";
import "./settings-tabbar.css";

export interface SettingsTabbarItem<Id extends string> {
  id: Id;
  label: string;
  icon?: ReactNode;
  /* Précision secondaire — affichée au survol et jointe au nom accessible.
     Elle reste hors du libellé visible, qui doit tenir sur une seule ligne. */
  hint?: string;
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
          aria-label={item.hint ? `${item.label} ${item.hint}` : undefined}
          title={item.hint}
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
