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
  providerEditable?: boolean;
}

export function WakeupModelFields({
  provider,
  model,
  providers,
  models,
  onProviderChange,
  onModelChange,
  providerEditable = true,
}: ModelFieldsProps) {
  const { t } = useTranslation();
  return (
    <div className="nwd-pair">
      <WakeupField label={t("heartbeat.form.provider")}>
        {providerEditable ? (
          <CustomSelect
            value={provider}
            onChange={onProviderChange}
            ariaLabel={t("heartbeat.form.provider")}
            options={providers.map((p) => ({ value: p.id, label: p.display_name }))}
          />
        ) : <span className="nwd-fixed-value">{provider}</span>}
      </WakeupField>

      <WakeupField label={t("heartbeat.form.model")}>
        <CustomSelect
          value={model}
          onChange={onModelChange}
          ariaLabel={t("heartbeat.form.model")}
          disabled={models.length === 0}
          placeholder={
            model || (models.length === 0
              ? t("heartbeat.form.noToolCapable")
              : t("heartbeat.form.pickModel"))
          }
          options={models.map((m) => ({ value: m.id, label: m.id }))}
        />
      </WakeupField>
    </div>
  );
}
