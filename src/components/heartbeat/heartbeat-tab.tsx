import { useState, useEffect, useMemo, memo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { useWakeups } from "@/hooks/use-wakeups";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import { formatDateTime, formatSchedule } from "@/lib/wakeup-format";
import { WakeupGrid } from "./wakeup-grid";
import { WakeupDetails } from "./wakeup-details";
import { NewWakeupDialog } from "./new-wakeup-dialog";
import type { ScheduledWakeup } from "@/types/wakeup";
import { ToggleSwitch } from "@/components/ui/toggle-switch";
import { PanelSlot } from "@/components/layout/panel-slots";
import "./heartbeat.css";

type DialogState =
  | { kind: "none" }
  | { kind: "create" }
  | { kind: "edit"; wakeup: ScheduledWakeup };

interface HeartbeatTabProps {
  activeWakeupId?: string | null;
  onWakeupChange?: (id: string | null) => void;
  listFocused?: boolean;
}

export const HeartbeatTab = memo(function HeartbeatTab({
  activeWakeupId,
  onWakeupChange,
  listFocused = true,
}: HeartbeatTabProps) {
  const { t } = useTranslation();
  const { wakeups, runs, summaries, globalPaused, setPaused, toggle, remove, create, update } = useWakeups();
  const [selectedId, setSelectedIdState] = useState<string | null>(null);

  useEffect(() => {
    if (activeWakeupId !== undefined) setSelectedIdState(activeWakeupId);
  }, [activeWakeupId]);

  const setSelectedId = useCallback((id: string | null) => {
    setSelectedIdState(id);
    onWakeupChange?.(id);
  }, [onWakeupChange]);
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  const selected = useMemo(
    () => selectedId ? wakeups.find((w) => w.id === selectedId) ?? null : null,
    [selectedId, wakeups],
  );

  const activeWakeups = useMemo(() => wakeups.filter((w) => w.active), [wakeups]);
  const wakeupIds = useMemo(() => activeWakeups.map((w) => w.id), [activeWakeups]);

  useArrowNavigation({
    items: wakeupIds,
    selectedId: selectedId,
    onSelect: setSelectedId,
    enabled: listFocused,
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  const handleDelete = useCallback(async () => {
    if (!selected) return;
    const id = selected.id;
    setSelectedId(null);
    await remove(id);
  }, [remove, selected, setSelectedId]);

  const list = useMemo(() => (
    <div className="wk-sidebar">
      <div className="wk-sidebar-header">
        <span className="wk-sidebar-title">{t("heartbeat.sidebar.title")}</span>
        <Tooltip label={globalPaused ? t("heartbeat.sidebar.resume") : t("heartbeat.sidebar.pause")} align="right">
          <ToggleSwitch
            checked={!globalPaused}
            ariaLabel={globalPaused ? t("heartbeat.sidebar.resume") : t("heartbeat.sidebar.pause")}
            onCheckedChange={(on) => void setPaused(!on)}
          />
        </Tooltip>
      </div>
      <div className="wk-sidebar-list">
        {activeWakeups.length === 0 ? (
          <div className="wk-sidebar-empty">
            {globalPaused
              ? t("heartbeat.sidebar.pausedEmpty")
              : t("heartbeat.sidebar.empty")}
          </div>
        ) : (
          activeWakeups.map((w) => (
            <button
              key={w.id}
              className={`wk-sidebar-item ${selectedId === w.id ? "active" : ""}`}
              data-nav-active={selectedId === w.id ? "true" : undefined}
              onClick={() => setSelectedId(w.id)}
              type="button"
            >
              <div className="wk-sidebar-model">{w.model}</div>
              <div className="wk-sidebar-schedule">{formatSchedule(w.schedule)}</div>
              <div className="wk-sidebar-next">{formatDateTime(summaries[w.id]?.next_fire_at)}</div>
            </button>
          ))
        )}
      </div>
    </div>
  ), [activeWakeups, globalPaused, setPaused, selectedId, setSelectedId, summaries, t]);

  const detail = useMemo(() => (
    <>
      {selected ? (
        <WakeupDetails
          wakeup={selected}
          summary={summaries[selected.id]}
          runs={runs.filter((run) => run.wakeup_id === selected.id)}
          disableToggle={globalPaused}
          onBack={() => setSelectedId(null)}
          onToggle={(active) => void toggle(selected.id, active)}
          onEdit={() => setDialog({ kind: "edit", wakeup: selected })}
          onDelete={() => void handleDelete()}
        />
      ) : (
          <WakeupGrid
            wakeups={wakeups}
            summaries={summaries}
            onSelect={setSelectedId}
          onCreate={() => setDialog({ kind: "create" })}
        />
      )}

      {dialog.kind !== "none" && (
        <NewWakeupDialog
          initial={dialog.kind === "edit" ? dialog.wakeup : null}
          onClose={() => setDialog({ kind: "none" })}
          onCreate={async (input) => { await create(input); }}
          onUpdate={async (w) => { await update(w); }}
        />
      )}
    </>
  ), [create, dialog, globalPaused, handleDelete, runs, selected, setSelectedId, summaries, toggle, update, wakeups]);

  return <><PanelSlot name="list">{list}</PanelSlot><PanelSlot name="detail">{detail}</PanelSlot></>;
});
