import "./update-progress-action.css";

interface UpdateProgressActionProps {
  percent: number;
  cancelling: boolean;
  cancelLabel: string;
  cancellingLabel: string;
  onCancel: () => void;
  compact?: boolean;
}

export function UpdateProgressAction({
  percent,
  cancelling,
  cancelLabel,
  cancellingLabel,
  onCancel,
  compact = false,
}: UpdateProgressActionProps) {
  const safePercent = Math.max(0, Math.min(100, percent));
  return (
    <div className={`upa-root ${compact ? "upa-compact" : ""}`}>
      <div className="upa-track" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={safePercent}>
        <div className="upa-fill" style={{ width: `${safePercent}%` }} />
      </div>
      <span className="upa-percent">{safePercent}%</span>
      <button type="button" className="btn btn-sm btn-destructive upa-cancel" disabled={cancelling} onClick={onCancel}>
        {cancelling ? cancellingLabel : cancelLabel}
      </button>
    </div>
  );
}
