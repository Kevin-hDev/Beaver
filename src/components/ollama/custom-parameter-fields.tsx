import type { TFunction } from "i18next";
import { Tooltip } from "@/components/ui/tooltip";
import {
  MAX_CUSTOM_PARAMETERS,
  MAX_PARAMETER_KEY_LENGTH,
  MAX_PARAMETER_VALUE_LENGTH,
} from "./parameter-editor-state";
import type { ModelParameter } from "./modelfile-utils";

interface CustomParameterFieldsProps {
  parameters: ModelParameter[];
  rowIds: string[];
  t: TFunction;
  onChange: (index: number, field: "key" | "value", value: string) => void;
  onAdd: () => void;
  onRemove: (index: number) => void;
}

export function CustomParameterFields({
  parameters,
  rowIds,
  t,
  onChange,
  onAdd,
  onRemove,
}: CustomParameterFieldsProps) {
  return (
    <section className="pe-custom-section">
      <h3 className="pe-group-title">{t("ollama.customParameters")}</h3>
      <p className="pe-custom-hint">{t("ollama.customParametersHint")}</p>
      {parameters.map((parameter, index) => (
        <div className="pe-custom-row" key={rowIds[index]}>
          <input
            value={parameter.key}
            onChange={(event) => onChange(index, "key", event.target.value)}
            placeholder={t("ollama.customParameterName")}
            aria-label={`${t("ollama.customParameterName")} ${index + 1}`}
            maxLength={MAX_PARAMETER_KEY_LENGTH}
            className="pe-input pe-custom-key"
          />
          <input
            value={parameter.value}
            onChange={(event) => onChange(index, "value", event.target.value)}
            placeholder={t("ollama.customParameterValue")}
            aria-label={`${t("ollama.customParameterValue")} ${index + 1}`}
            maxLength={MAX_PARAMETER_VALUE_LENGTH}
            className="pe-input pe-custom-value"
          />
          <Tooltip label={t("ollama.remove")}>
            <button
              type="button"
              className="ollama-btn pe-clear-btn"
              aria-label={t("ollama.remove")}
              onClick={() => onRemove(index)}
            >
              ×
            </button>
          </Tooltip>
        </div>
      ))}
      <button
        type="button"
        className="ollama-btn pe-add-btn"
        onClick={onAdd}
        disabled={parameters.length >= MAX_CUSTOM_PARAMETERS}
      >
        {t("ollama.addCustomParameter")}
      </button>
    </section>
  );
}
