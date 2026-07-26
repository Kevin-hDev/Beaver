import { useId } from "react";

interface FieldSelectProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: string[];
}

interface OptionalFieldSelectProps extends FieldSelectProps {
  emptyLabel: string;
}

export function FieldSelect({
  label,
  value,
  onChange,
  options,
}: FieldSelectProps) {
  const id = useId();
  return (
    <div className="fcc-field">
      <label className="fcc-label" htmlFor={id}>{label}</label>
      <select className="fcc-select" id={id} value={value} onChange={(e) => onChange(e.target.value)}>
        {options.map((option) => <option key={option} value={option}>{option}</option>)}
      </select>
    </div>
  );
}

export function OptionalFieldSelect({
  label,
  value,
  onChange,
  options,
  emptyLabel,
}: OptionalFieldSelectProps) {
  const id = useId();
  return (
    <div className="fcc-field">
      <label className="fcc-label" htmlFor={id}>{label}</label>
      <select className="fcc-select" id={id} value={value} onChange={(e) => onChange(e.target.value)}>
        <option value="">{emptyLabel}</option>
        {options.map((option) => <option key={option} value={option}>{option}</option>)}
      </select>
    </div>
  );
}
