import { memo, useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { PanelSlot } from "@/components/ui/panel-slots";
import { useWakeups } from "@/hooks/use-wakeups";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import { formatDateTime, formatSchedule } from "@/lib/wakeup-format";
import { WakeupList } from "./wakeup-list";
import { WakeupDetails } from "./wakeup-details";
import { NewWakeupDialog } from "./new-wakeup-dialog";
import type { WakeupDefinition } from "@/types/wakeup";
import "./heartbeat.css";

interface HeartbeatTabProps {
  activeWakeupId?: string | null;
  onWakeupChange?: (id: string | null) => void;
  listFocused?: boolean;
}

export const HeartbeatTab = memo(function HeartbeatTab({
  activeWakeupId, onWakeupChange, listFocused = true,
}: HeartbeatTabProps) {
  const { t } = useTranslation();
  const wakeupsApi = useWakeups();
  const [selectedId, setSelectedIdState] = useState<string | null>(null);
  const [dialog, setDialog] = useState<"none" | "create" | "edit">("none");
  const [timezone, setTimezone] = useState(Intl.DateTimeFormat().resolvedOptions().timeZone);
  const timezones = useMemo(() => {
    const supported = (Intl as typeof Intl & { supportedValuesOf?: (key: "timeZone") => string[] }).supportedValuesOf;
    return supported?.("timeZone") ?? [];
  }, []);

  useEffect(() => {
    if (activeWakeupId !== undefined) setSelectedIdState(activeWakeupId);
  }, [activeWakeupId]);

  const select = useCallback((id: string | null) => {
    setSelectedIdState(id);
    onWakeupChange?.(id);
    if (id) void wakeupsApi.loadDetail(id);
  }, [onWakeupChange, wakeupsApi]);
  const selected = wakeupsApi.wakeups.find((item) => item.id === selectedId) ?? null;

  useArrowNavigation({
    items: wakeupsApi.wakeups.map((item) => item.id), selectedId, onSelect: select,
    enabled: listFocused, focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const removeSelected = useCallback(async () => {
    if (!selected) return;
    await wakeupsApi.remove(selected.id);
    select(null);
  }, [selected, select, wakeupsApi]);

  const sidebar = useMemo(() => (
    <div className="wk-sidebar">
      <div className="wk-sidebar-header">
        <span className="wk-sidebar-title">{t("heartbeat.sidebar.title")}</span>
        <Tooltip label={wakeupsApi.globalPaused ? t("heartbeat.sidebar.resume") : t("heartbeat.sidebar.pause")} align="right">
          <ToggleSwitch checked={!wakeupsApi.globalPaused} ariaLabel={t("heartbeat.sidebar.pause")} onCheckedChange={(enabled) => void wakeupsApi.setPaused(!enabled)} />
        </Tooltip>
      </div>
      <div className="wk-sidebar-list">
        {wakeupsApi.wakeups.map((item) => (
          <button key={item.id} className={`wk-sidebar-item ${selectedId === item.id ? "active" : ""}`} data-nav-active={selectedId === item.id ? "true" : undefined} onClick={() => select(item.id)} type="button">
            <div className="wk-sidebar-model">{item.name}</div>
            <div className="wk-sidebar-schedule">{formatSchedule(item.schedule)}</div>
            <div className="wk-sidebar-next">{formatDateTime(item.next_fire_at)}</div>
          </button>
        ))}
      </div>
    </div>
  ), [selectedId, select, t, wakeupsApi]);

  if (wakeupsApi.migration?.status === "needs_timezone") {
    return <PanelSlot name="detail"><div className="wk-blocking"><h2>{t("heartbeat.migration.title")}</h2><p>{t("heartbeat.migration.timezone")}</p><input className="field" list="wk-timezones" value={timezone} onChange={(event) => setTimezone(event.target.value)} /><datalist id="wk-timezones">{timezones.map((zone) => <option key={zone} value={zone} />)}</datalist><button className="btn btn-primary" type="button" onClick={() => void wakeupsApi.chooseTimezone(timezone)}>{t("heartbeat.migration.continue")}</button></div></PanelSlot>;
  }

  const definition: WakeupDefinition | null = wakeupsApi.detail?.definition ?? null;
  const detail = (
    <>
      {wakeupsApi.error && <div className="wk-alert" role="alert">{t(`heartbeat.errors.${wakeupsApi.error}`)}{wakeupsApi.error === "audit_unavailable" && <button className="btn btn-sm btn-secondary" type="button" onClick={() => void wakeupsApi.setPaused(true)}>{t("heartbeat.errors.pauseAll")}</button>}</div>}
      {wakeupsApi.migration?.status === "conflicts" && <div className="wk-alert" role="alert"><span>{t("heartbeat.migration.conflicts")}</span>{wakeupsApi.migration.conflicts.map((conflict) => <span key={conflict.legacy_id} className="wk-conflict"><code>{conflict.legacy_id}</code><button className="btn btn-sm btn-secondary" type="button" onClick={() => void wakeupsApi.resolveConflict(conflict.legacy_id, "remove_historical")}>{t("heartbeat.migration.remove")}</button><button className="btn btn-sm btn-secondary" type="button" onClick={() => void wakeupsApi.resolveConflict(conflict.legacy_id, "import_as_new", Intl.DateTimeFormat().resolvedOptions().timeZone)}>{t("heartbeat.migration.import")}</button></span>)}</div>}
      {selected ? <WakeupDetails summary={selected} detail={wakeupsApi.detail} runs={wakeupsApi.history.entries} hasMore={Boolean(wakeupsApi.history.next_cursor)} loading={wakeupsApi.detailLoading} onBack={() => select(null)} onToggle={(active) => void wakeupsApi.toggle(selected.id, active)} onEdit={() => setDialog("edit")} onDelete={() => void removeSelected()} onLoadMore={() => void wakeupsApi.loadMoreHistory()} /> : <WakeupList wakeups={wakeupsApi.wakeups} onSelect={select} onCreate={() => setDialog("create")} />}
      {dialog !== "none" && <NewWakeupDialog initial={dialog === "edit" ? definition : null} onClose={() => setDialog("none")} onCreate={wakeupsApi.create} onUpdate={wakeupsApi.update} />}
    </>
  );

  return <><PanelSlot name="list">{sidebar}</PanelSlot><PanelSlot name="detail">{detail}</PanelSlot></>;
});
