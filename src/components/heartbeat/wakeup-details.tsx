import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretLeft, Pencil, Trash } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { SettingsCard } from "@/components/settings/settings-card";
import { SettingsRow } from "@/components/settings/settings-row";
import type { ScheduledWakeup, WakeupDetail, WakeupRun } from "@/types/wakeup";
import { displayStatus } from "@/types/wakeup";
import { formatDateTime, formatRunStatus, formatSchedule, formatTarget } from "@/lib/wakeup-format";
import { RunErrorBadge, StatusBadge } from "./badges";
import { WakeupHistory } from "./wakeup-history";

interface WakeupDetailsProps {
  summary: ScheduledWakeup;
  detail: WakeupDetail | null;
  runs: WakeupRun[];
  hasMore: boolean;
  loading: boolean;
  onBack: () => void;
  onToggle: (active: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
  onLoadMore: () => void;
}

export function WakeupDetails({
  summary, detail, runs, hasMore, loading, onBack, onToggle, onEdit, onDelete, onLoadMore,
}: WakeupDetailsProps) {
  const { t } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const definition = detail?.definition;

  return (
    <div className="wk-details">
      <div className="wk-details-header">
        <Tooltip label={t("heartbeat.back")}><button className="wk-back" onClick={onBack} type="button"><CaretLeft size="var(--icon-sm)" /></button></Tooltip>
        <div className="wk-details-title"><span className="wk-details-model">{summary.name}</span><RunErrorBadge run={summary.last_run} /><StatusBadge status={displayStatus(summary)} /></div>
        <div className="wk-details-actions">
          <Tooltip label={t("heartbeat.edit")}><button className="icon-btn" onClick={onEdit} disabled={!definition} type="button"><Pencil size="var(--icon-sm)" /></button></Tooltip>
          {confirmDelete ? (
            <button className="wk-confirm-delete" onClick={onDelete} type="button"><Trash size="var(--icon-sm)" />{t("heartbeat.confirmDelete")}</button>
          ) : (
            <Tooltip label={t("heartbeat.delete")}><button className="icon-btn icon-btn-destructive" onClick={() => setConfirmDelete(true)} type="button"><Trash size="var(--icon-sm)" /></button></Tooltip>
          )}
          <ToggleSwitch checked={summary.status === "active"} disabled={summary.paused_by_global} ariaLabel={t("heartbeat.toggle")} title={summary.paused_by_global ? t("heartbeat.pausedHint") : undefined} onCheckedChange={onToggle} />
        </div>
      </div>
      <div className="wk-details-body">
        {loading && <div className="wk-empty">{t("common.loading")}</div>}
        <SettingsCard>
          <SettingsRow title={t("heartbeat.fields.model")}><span className="wk-row-value">{summary.model}</span></SettingsRow>
          <SettingsRow title={t("heartbeat.fields.provider")}><span className="wk-row-value">{summary.provider}</span></SettingsRow>
          <SettingsRow title={t("heartbeat.fields.target")}><span className="wk-row-value">{formatTarget(summary.target)}</span></SettingsRow>
          <SettingsRow title={t("heartbeat.fields.schedule")}><span className="wk-row-value">{formatSchedule(summary.schedule)}</span></SettingsRow>
          <SettingsRow title={t("heartbeat.fields.nextFire")}><span className="wk-row-value">{formatDateTime(summary.next_fire_at)}</span></SettingsRow>
          <SettingsRow title={t("heartbeat.fields.lastStatus")}><span className="wk-row-value">{formatRunStatus(summary.last_run?.status)}</span></SettingsRow>
        </SettingsCard>
        {definition && <SettingsCard><div className="wk-prompt"><div className="wk-prompt-title">{t("heartbeat.fields.prompt")}</div><div className="wk-prompt-text">{definition.prompt}</div></div></SettingsCard>}
        <WakeupHistory runs={runs} hasMore={hasMore} onLoadMore={onLoadMore} />
      </div>
    </div>
  );
}
