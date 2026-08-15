import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretLeft, Pencil, Trash } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsCard } from "@/components/settings/settings-card";
import { SettingsRow } from "@/components/settings/settings-row";
import type { ScheduledWakeup, WakeupRun, WakeupStatusSummary } from "@/types/wakeup";
import { formatDateTime, formatRunStatus, formatSchedule } from "@/lib/wakeup-format";
import { ActiveBadge } from "./badges";
import { WakeupHistory } from "./wakeup-history";

interface WakeupDetailsProps {
  wakeup: ScheduledWakeup;
  summary?: WakeupStatusSummary;
  runs: WakeupRun[];
  disableToggle: boolean;
  onBack: () => void;
  onToggle: (active: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
}

export function WakeupDetails({
  wakeup,
  summary,
  runs,
  disableToggle,
  onBack,
  onToggle,
  onEdit,
  onDelete,
}: WakeupDetailsProps) {
  const { t } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const lastRun = summary?.last_run ?? null;

  const handleDelete = () => {
    if (confirmDelete) {
      onDelete();
    } else {
      setConfirmDelete(true);
      window.setTimeout(() => setConfirmDelete(false), 3000);
    }
  };

  return (
    <div className="wk-details">
      <div className="wk-details-header">
        <Tooltip label={t("heartbeat.back")}>
          <button className="wk-back" onClick={onBack} type="button">
            <CaretLeft size="var(--icon-sm)" weight="regular" />
          </button>
        </Tooltip>
        <div className="wk-details-title">
          <span className="wk-details-model">{wakeup.model}</span>
          {wakeup.provider !== "ollama" && (
            <span className="wk-provider-tag">{wakeup.provider}</span>
          )}
          <ActiveBadge active={wakeup.active} />
        </div>
        <div className="wk-details-actions">
          <Tooltip label={t("heartbeat.edit")}>
            <button
              className="icon-btn"
              onClick={onEdit}
              type="button"
            >
              <Pencil size="var(--icon-sm)" />
            </button>
          </Tooltip>
          {confirmDelete ? (
            <button
              className="wk-confirm-delete"
              onClick={handleDelete}
              type="button"
            >
              <Trash size="var(--icon-sm)" />
              {t("heartbeat.confirmDelete")}
            </button>
          ) : (
            <Tooltip label={t("heartbeat.delete")}>
              <button
                className="icon-btn icon-btn-destructive"
                onClick={handleDelete}
                type="button"
              >
                <Trash size="var(--icon-sm)" />
              </button>
            </Tooltip>
          )}
          <Tooltip
            label={disableToggle ? t("heartbeat.pausedHint") : t("heartbeat.toggle")}
            align="right"
          >
            <ToggleSwitch
              checked={wakeup.active}
              ariaLabel={t("heartbeat.toggle")}
              disabled={disableToggle}
              onCheckedChange={onToggle}
            />
          </Tooltip>
        </div>
      </div>

      <div className="wk-details-body">
        {/* Une ligne par information : le nom du réveil n'apparaissait nulle
            part, et le fournisseur était répété en légende de deux lignes. */}
        <SettingsCard>
          <SettingsRow title={t("heartbeat.fields.name")}>
            <span className="wk-row-value">{wakeup.name || "—"}</span>
          </SettingsRow>
          <SettingsRow title={t("heartbeat.fields.model")}>
            <span className="wk-row-value">{wakeup.model}</span>
          </SettingsRow>
          <SettingsRow title={t("heartbeat.fields.provider")}>
            <span className="wk-row-value">{wakeup.provider}</span>
          </SettingsRow>
          <SettingsRow title={t("heartbeat.fields.description")}>
            <span className="wk-row-value">{wakeup.description || "—"}</span>
          </SettingsRow>
        </SettingsCard>

        <SettingsCard>
          <SettingsRow title={t("heartbeat.fields.schedule")}>
            <span className="wk-row-value">{formatSchedule(wakeup.schedule)}</span>
          </SettingsRow>
          <SettingsRow title={t("heartbeat.fields.nextFire")}>
            <span className="wk-row-value">{formatDateTime(summary?.next_fire_at) || "—"}</span>
          </SettingsRow>
          <SettingsRow title={t("heartbeat.fields.lastStatus")}>
            <span className="wk-row-value">{formatRunStatus(lastRun?.status) || "—"}</span>
          </SettingsRow>
          <SettingsRow title={t("heartbeat.fields.lastRun")}>
            <span className="wk-row-value">{formatDateTime(lastRun?.fired_at) || "—"}</span>
          </SettingsRow>
        </SettingsCard>

        {/* Le prompt tient sur plusieurs lignes : en valeur de ligne de réglage,
            il se posait par-dessus son propre libellé et débordait à droite. */}
        <SettingsCard>
          <div className="wk-prompt">
            <div className="wk-prompt-title">{t("heartbeat.fields.prompt")}</div>
            <div className="wk-prompt-text">{wakeup.prompt}</div>
          </div>
        </SettingsCard>

        <WakeupHistory runs={runs} />
      </div>
    </div>
  );
}
