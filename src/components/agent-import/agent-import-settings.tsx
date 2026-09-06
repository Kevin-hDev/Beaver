import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";
import { SettingsRow } from "@/components/settings/settings-row";
import { DialogPortal } from "@/components/ui/dialog-portal";
import { useKeyboard } from "@/hooks/use-keyboard";
import { AgentImportWizard } from "./agent-import-wizard";
import "./agent-import-dialog.css";

export function AgentImportSettings() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const close = useCallback(() => setOpen(false), []);
  useKeyboard({ onEscape: open ? close : undefined });

  return (
    <>
      <SettingsCard>
        <SettingsRow
          title={t("agentImport.settings.title")}
          description={t("agentImport.settings.description")}
        >
          <button
            type="button"
            className="btn btn-sm btn-secondary"
            onClick={() => setOpen(true)}
          >
            {t("agentImport.settings.manage")}
          </button>
        </SettingsRow>
      </SettingsCard>

      {/* Le calque sort de la page : celle des Réglages porte le fondu sous son
          titre figé, et un masque force WebKit à peindre tout le sous-arbre dans
          un tampon aux dimensions du panneau — le dialogue y était découpé au
          bord de la barre latérale, effacé en haut, et la molette remontait
          jusqu'à la page derrière. */}
      {open && (
        <DialogPortal>
          <div
            className="aim-dialog-backdrop"
            role="presentation"
            onMouseDown={(event) => {
              if (event.target === event.currentTarget) close();
            }}
          >
            <div
              className="aim-dialog"
              role="dialog"
              aria-modal="true"
              aria-label={t("agentImport.title")}
            >
              <AgentImportWizard onClose={close} />
            </div>
          </div>
        </DialogPortal>
      )}
    </>
  );
}
