import type { ReactNode } from "react";
import { cn } from "@/lib/utils";
import "./settings-panel.css";

interface SettingsPanelProps {
  title?: string;
  action?: ReactNode;
  /** Élargit la colonne pour les pages dont les lignes portent icône et
      commandes, à l'étroit dans la largeur standard. */
  wide?: boolean;
  children: ReactNode;
}

/**
 * Page de Réglages : un titre figé en haut, et le contenu qui défile dessous en
 * s'effaçant sous lui. Le titre vit hors de la zone qui défile — un masque
 * s'applique à un élément et à toute sa descendance, il effacerait le titre
 * avec le reste.
 */
export function SettingsPanel({ title, action, wide = false, children }: SettingsPanelProps) {
  const hasHeader = title !== undefined || action !== undefined;
  const column = cn("settings-panel-inner", wide && "settings-panel-inner-wide");
  return (
    <div className={cn("settings-panel", hasHeader && "settings-panel-headed")}>
      <div className="settings-panel-scroll">
        <div className={column}>{children}</div>
      </div>
      {hasHeader && (
        <header className="settings-panel-header">
          <div className={column}>
            <div className="settings-panel-header-row">
              {title !== undefined && <h2 className="settings-panel-title">{title}</h2>}
              {action}
            </div>
          </div>
        </header>
      )}
    </div>
  );
}
