interface CompressionQuantityControlProps {
  value: number;
  minimum?: number;
  maximum: number;
  disabled: boolean;
  ariaLabel: string;
  unit?: string;
  onChange: (value: number) => void;
}

export function CompressionQuantityControl({
  value,
  minimum = 0,
  maximum,
  disabled,
  ariaLabel,
  unit,
  onChange,
}: CompressionQuantityControlProps) {
  return (
    <>
      <input
        className="field cse-num"
        type="number"
        min={minimum}
        max={maximum}
        value={value}
        disabled={disabled}
        aria-label={ariaLabel}
        onChange={(event) => {
          const parsed = Number.parseInt(event.target.value, 10);
          const next = Number.isFinite(parsed) ? parsed : minimum;
          onChange(Math.min(maximum, Math.max(minimum, next)));
        }}
      />
      {unit && <span className="cse-row-unit">{unit}</span>}
    </>
  );
}
