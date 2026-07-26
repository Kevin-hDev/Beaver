import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";
import { SettingsRow } from "@/components/settings/settings-row";
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
            className="aim-btn aim-btn-secondary"
            onClick={() => setOpen(true)}
          >
            {t("agentImport.settings.manage")}
          </button>
        </SettingsRow>
      </SettingsCard>

      {open && (
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
      )}
    </>
  );
}
