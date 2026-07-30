import type { ReactNode } from "react";
import { CaretRight } from "@/components/ui/icons";
import { EmptyState } from "@/components/ui/empty-state";
import { SettingsCard } from "../settings-card";
import "./settings-entry-list.css";

export interface SettingsEntry {
  id: string;
  label: string;
  description?: string;
  icon?: ReactNode;
  /** Badge, coche ou compteur affiché juste avant le chevron. */
  trailing?: ReactNode;
  /** Libellé de l'état hors service : affiche une pastille et complète le nom
      accessible du bouton, qu'une pastille seule ne dit pas. */
  offlineLabel?: string;
}

interface SettingsEntryListProps {
  entries: ReadonlyArray<SettingsEntry>;
  emptyMessage: string;
  onSelect: (id: string) => void;
}

export function SettingsEntryList({ entries, emptyMessage, onSelect }: SettingsEntryListProps) {
  if (entries.length === 0) {
    return (
      <div className="settings-entry-empty">
        <EmptyState message={emptyMessage} />
      </div>
    );
  }

  return (
    <SettingsCard>
      {entries.map((entry) => (
        <button
          key={entry.id}
          type="button"
          className="settings-entry"
          aria-label={entry.offlineLabel ? `${entry.label} — ${entry.offlineLabel}` : undefined}
          onClick={() => onSelect(entry.id)}
        >
          {entry.icon && <span className="settings-entry-icon">{entry.icon}</span>}
          <span className="settings-entry-text">
            <span className="settings-entry-label">{entry.label}</span>
            {entry.description && (
              <span className="settings-entry-description">{entry.description}</span>
            )}
          </span>
          {entry.trailing && <span className="settings-entry-trailing">{entry.trailing}</span>}
          {entry.offlineLabel && <span className="settings-entry-dot" />}
          <CaretRight size="var(--icon-md)" className="settings-entry-caret" />
        </button>
      ))}
    </SettingsCard>
  );
}
