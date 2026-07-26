import { SettingsSelect, type SelectOption } from "./settings-select";

interface Props {
  options: SelectOption[];
  value: string;
  changed: boolean;
  restarting: boolean;
  onSelect: (v: string) => void;
  onRestart: () => void;
  restartLabel: string;
}

export function HardwareAccelControl({
  options, value, changed, restarting, onSelect, onRestart, restartLabel,
}: Props) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
      <SettingsSelect options={options} value={value} onChange={onSelect} />
      {changed && (
        <button disabled={restarting} onClick={onRestart} style={{
          padding: "6px 12px", borderRadius: "var(--radius-md)", border: "none",
          background: "var(--accent)", color: "var(--ink-on-pulse)", fontSize: "var(--text-xs)",
          cursor: restarting ? "wait" : "pointer", whiteSpace: "nowrap",
          opacity: restarting ? 0.6 : 1,
        }}>
          {restartLabel}
        </button>
      )}
    </div>
  );
}
