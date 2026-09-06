import { cn } from "@/lib/utils";
import "./operation-progress-action.css";

export interface OperationProgressActionProps {
  percent: number | null;
  phaseLabel?: string;
  cancelling: boolean;
  canCancel: boolean;
  cancelLabel: string;
  cancellingLabel: string;
  onCancel: () => void;
  compact?: boolean;
}

export function OperationProgressAction({
  percent, phaseLabel, cancelling, canCancel, cancelLabel, cancellingLabel,
  onCancel, compact = false,
}: OperationProgressActionProps) {
  const safePercent = percent !== null && Number.isFinite(percent)
    ? Math.max(0, Math.min(100, percent)) : null;
  return (
    <div className={cn("opa-root", compact && "opa-compact")}>
      <div className="opa-track" role="progressbar" aria-label={phaseLabel}
        aria-valuemin={0} aria-valuemax={100} aria-valuenow={safePercent ?? undefined}>
        <div className={cn("opa-fill", safePercent === null && "operation-progress-indeterminate",
          cancelling && "opa-stopped")}
          style={safePercent === null ? undefined : { width: `${safePercent}%` }} />
      </div>
      <span className="opa-percent">{cancelling ? cancellingLabel : safePercent === null ? phaseLabel : `${safePercent}%`}</span>
      {(canCancel || cancelling) && (
        <button type="button" className="btn btn-sm btn-destructive opa-cancel"
          disabled={cancelling} onClick={onCancel}>
          {cancelling ? cancellingLabel : cancelLabel}
        </button>
      )}
    </div>
  );
}
