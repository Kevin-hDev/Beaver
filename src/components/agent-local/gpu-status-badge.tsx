import { useRef, useState, useCallback } from "react";
import { createPortal } from "react-dom";
import { useGpuStatus } from "@/hooks/use-gpu-status";
import { useSettingValue } from "@/hooks/use-setting-value";
import "./gpu-status-badge.css";

function formatMb(mb: number): string {
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${mb} MB`;
}

export function GpuStatusBadge() {
  const showGpuStatus = useSettingValue<boolean>("show_gpu_status", false);
  const gpu = useGpuStatus();
  const badgeRef = useRef<HTMLSpanElement>(null);
  const [tipPos, setTipPos] = useState<{ top: number; right: number } | null>(null);

  const showTip = useCallback(() => {
    const el = badgeRef.current;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    setTipPos({ top: rect.top - 28, right: window.innerWidth - rect.right });
  }, []);

  const hideTip = useCallback(() => setTipPos(null), []);

  if (!showGpuStatus || !gpu.accelerator) return null;

  const pct = gpu.vramPercent;
  const label = pct > 0
    ? `${gpu.accelerator} ${pct}%`
    : gpu.vramUsedMb > 0
      ? `${gpu.accelerator} ${formatMb(gpu.vramUsedMb)}`
      : gpu.accelerator;
  const isHigh = pct >= 85;

  const usedStr = gpu.vramUsedMb > 0 ? formatMb(gpu.vramUsedMb) : "—";
  const totalStr = gpu.vramTotalMb > 0 ? formatMb(gpu.vramTotalMb) : "—";
  const tooltip = gpu.modelLoaded
    ? `${gpu.accelerator} — ${usedStr} / ${totalStr} (${gpu.modelLoaded})`
    : `${gpu.accelerator} — ${usedStr} / ${totalStr}`;

  return (
    <span
      ref={badgeRef}
      className={`gpu-badge ${isHigh ? "gpu-badge-high" : ""}`}
      onMouseEnter={showTip}
      onMouseLeave={hideTip}
    >
      {label}
      {tipPos && createPortal(
        <span
          className="gpu-badge-tooltip"
          style={{ top: tipPos.top, right: tipPos.right }}
        >
          {tooltip}
        </span>,
        document.body,
      )}
    </span>
  );
}
