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
  const commit = (input: HTMLInputElement) => {
    const parsed = Number.parseInt(input.value, 10);
    const next = Number.isFinite(parsed) ? parsed : minimum;
    const bounded = Math.min(maximum, Math.max(minimum, next));
    input.value = String(bounded);
    if (bounded !== value) onChange(bounded);
  };

  return (
    <>
      <input
        key={`${ariaLabel}-${value}`}
        className="field cse-num"
        type="number"
        min={minimum}
        max={maximum}
        defaultValue={value}
        disabled={disabled}
        aria-label={ariaLabel}
        onChange={(event) => {
          const parsed = Number.parseInt(event.target.value, 10);
          if (Number.isFinite(parsed) && parsed >= minimum && parsed <= maximum) {
            onChange(parsed);
          }
        }}
        onBlur={(event) => commit(event.currentTarget)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            commit(event.currentTarget);
          }
        }}
      />
      {unit && <span className="cse-row-unit">{unit}</span>}
    </>
  );
}
