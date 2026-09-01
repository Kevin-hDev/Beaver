import { useMemo } from "react";
import { CustomSelect } from "@/components/ui/custom-select";

interface FieldSelectProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: string[];
}

interface OptionalFieldSelectProps extends FieldSelectProps {
  emptyLabel: string;
}

/* Le libellé n'est plus un `<label for>` : la liste maison est un bouton, et un
   `for` qui ne désigne aucun champ ne fait rien. C'est `ariaLabel` qui porte le
   nom pour les lecteurs d'écran. */
export function FieldSelect({ label, value, onChange, options }: FieldSelectProps) {
  const items = useMemo(
    () => options.map((option) => ({ value: option, label: option })),
    [options],
  );
  return (
    <div className="fcc-field">
      <span className="fcc-label">{label}</span>
      <CustomSelect options={items} value={value} onChange={onChange} ariaLabel={label} />
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
  const items = useMemo(
    () => [
      { value: "", label: emptyLabel },
      ...options.map((option) => ({ value: option, label: option })),
    ],
    [emptyLabel, options],
  );
  return (
    <div className="fcc-field">
      <span className="fcc-label">{label}</span>
      <CustomSelect
        options={items}
        value={value}
        onChange={onChange}
        placeholder={emptyLabel}
        ariaLabel={label}
      />
    </div>
  );
}
