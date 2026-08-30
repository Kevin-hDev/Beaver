import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { CustomSelect } from "@/components/ui/custom-select";
import type { QwenConnectionInput, QwenEndpointMode, QwenRegion } from "@/types/api";
import "./provider-connection-form.css";

const REGIONS: QwenRegion[] = [
  "beijing", "singapore", "hong_kong", "tokyo", "frankfurt", "virginia",
];

const MODES_BY_REGION: Record<QwenRegion, QwenEndpointMode[]> = {
  beijing: ["shared", "workspace", "trial"],
  singapore: ["shared", "workspace", "trial"],
  hong_kong: ["shared", "workspace", "trial"],
  tokyo: ["workspace"],
  frankfurt: ["workspace"],
  virginia: ["shared", "workspace"],
};

export const DEFAULT_QWEN_CONNECTION: QwenConnectionInput = {
  region: "singapore",
  endpointMode: "shared",
};

export function isQwenConnectionValid(value: QwenConnectionInput): boolean {
  // Miroir UI de `provider_connections/workspace_id.rs` ; Rust reste l'autorité.
  if (!MODES_BY_REGION[value.region].includes(value.endpointMode)) return false;
  if (value.endpointMode !== "workspace") return value.workspaceId === undefined;
  const workspace = value.workspaceId ?? "";
  return workspace.length >= 1
    && workspace.length <= 64
    && !workspace.startsWith("-")
    && !workspace.endsWith("-")
    && [...workspace].every((character) => (
      (character >= "a" && character <= "z")
      || (character >= "0" && character <= "9")
      || character === "-"
    ));
}

interface ProviderConnectionFormProps {
  value: QwenConnectionInput;
  onChange: (value: QwenConnectionInput) => void;
  disabled?: boolean;
}

export function ProviderConnectionForm({
  value,
  onChange,
  disabled = false,
}: ProviderConnectionFormProps) {
  const { t } = useTranslation();
  const modes = MODES_BY_REGION[value.region];
  const regionOptions = useMemo(() => REGIONS.map((region) => ({
    value: region,
    label: t(`apiKeys.connection.regions.${region}`),
  })), [t]);
  const modeOptions = modes.map((mode) => ({
    value: mode,
    label: t(`apiKeys.connection.modes.${mode}`),
  }));

  return (
    <fieldset className="pcf-fields" disabled={disabled}>
      <p className="pcf-help">{t("apiKeys.connection.payAsYouGo")}</p>
      <label className="pcf-field">
        <span className="wk-form-label">{t("apiKeys.connection.region")}</span>
        <CustomSelect
          value={value.region}
          options={regionOptions}
          ariaLabel={t("apiKeys.connection.region")}
          disabled={disabled}
          onChange={(region) => {
            const nextRegion = region as QwenRegion;
            const available = MODES_BY_REGION[nextRegion];
            const endpointMode = available.includes(value.endpointMode)
              ? value.endpointMode
              : available[0];
            onChange({
              region: nextRegion,
              endpointMode,
              ...(endpointMode === "workspace" ? { workspaceId: value.workspaceId } : {}),
            });
          }}
        />
      </label>
      <label className="pcf-field">
        <span className="wk-form-label">{t("apiKeys.connection.endpointMode")}</span>
        <CustomSelect
          value={value.endpointMode}
          options={modeOptions}
          ariaLabel={t("apiKeys.connection.endpointMode")}
          disabled={disabled}
          onChange={(endpointMode) => onChange({
            region: value.region,
            endpointMode: endpointMode as QwenEndpointMode,
            ...(endpointMode === "workspace" ? { workspaceId: value.workspaceId } : {}),
          })}
        />
      </label>
      {value.endpointMode === "workspace" && (
        <label className="pcf-field">
          <span className="wk-form-label">{t("apiKeys.connection.workspaceId")}</span>
          <input
            className="field wk-input"
            value={value.workspaceId ?? ""}
            maxLength={64}
            aria-label={t("apiKeys.connection.workspaceId")}
            onChange={(event) => onChange({ ...value, workspaceId: event.target.value })}
          />
        </label>
      )}
      <p className="pcf-note">{t("apiKeys.connection.unsupportedPlans")}</p>
    </fieldset>
  );
}
