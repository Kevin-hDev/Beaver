import { useTranslation } from "react-i18next";
import { CustomSelect } from "@/components/ui/custom-select";

interface FieldProps {
  label: string;
  required?: boolean;
  children: React.ReactNode;
}

/* Libellé en casse normale, avec la marque des champs obligatoires. Le style
   partagé des dialogues met ses libellés en capitales espacées ; il sert à
   dix-huit autres dialogues et n'est donc pas touché ici. */
export function WakeupField({ label, required, children }: FieldProps) {
  const { t } = useTranslation();
  return (
    <label className="nwd-field">
      <span className="nwd-label">
        {label}
        {required && (
          <span className="nwd-required" title={t("heartbeat.form.required")} aria-hidden="true">*</span>
        )}
      </span>
      {children}
    </label>
  );
}

interface ModelFieldsProps {
  provider: string;
  model: string;
  providers: { id: string; display_name: string }[];
  models: { id: string }[];
  onProviderChange: (value: string) => void;
  onModelChange: (value: string) => void;
}

export function WakeupModelFields({
  provider,
  model,
  providers,
  models,
  onProviderChange,
  onModelChange,
}: ModelFieldsProps) {
  const { t } = useTranslation();
  return (
    <div className="nwd-pair">
      <WakeupField label={t("heartbeat.form.provider")}>
        <CustomSelect
          value={provider}
          onChange={onProviderChange}
          options={
            providers.length === 0
              ? [{ value: "ollama", label: "Ollama" }]
              : providers.map((p) => ({ value: p.id, label: p.display_name }))
          }
        />
      </WakeupField>

      <WakeupField label={t("heartbeat.form.model")}>
        <CustomSelect
          value={model}
          onChange={onModelChange}
          disabled={models.length === 0}
          placeholder={
            models.length === 0
              ? t("heartbeat.form.noToolCapable")
              : t("heartbeat.form.pickModel")
          }
          options={models.map((m) => ({ value: m.id, label: m.id }))}
        />
      </WakeupField>
    </div>
  );
}
