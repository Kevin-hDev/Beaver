import { cn } from "@/lib/utils";
import "./toggle-switch.css";

interface ToggleSwitchProps {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  ariaLabel: string;
  className?: string;
  disabled?: boolean;
  id?: string;
  title?: string;
}

export function ToggleSwitch({
  checked,
  onCheckedChange,
  ariaLabel,
  className,
  disabled = false,
  id,
  title,
}: ToggleSwitchProps) {
  return (
    <label
      className={cn("uis-switch", disabled && "uis-switch-disabled", className)}
      title={title}
    >
      <input
        className="uis-input"
        id={id}
        type="checkbox"
        role="switch"
        aria-label={ariaLabel}
        checked={checked}
        disabled={disabled}
        onChange={(event) => onCheckedChange(event.target.checked)}
      />
      <span className="uis-slider" aria-hidden="true" />
    </label>
  );
}
