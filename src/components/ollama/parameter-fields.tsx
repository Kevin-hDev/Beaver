import { useId } from "react";
import type { TFunction } from "i18next";
import { Tooltip } from "@/components/ui/tooltip";
import {
  MAX_PARAMETER_VALUE_LENGTH,
  MAX_STOP_SEQUENCES,
} from "./parameter-editor-state";
import type { ModelParameterDefinition } from "./model-parameter-catalog";

interface ParameterFieldProps {
  definition: ModelParameterDefinition;
  value: string;
  t: TFunction;
  onChange: (value: string) => void;
}

export function ParameterField({ definition, value, t, onChange }: ParameterFieldProps) {
  const descriptionId = useId();
  const isInteger = definition.valueType === "integer";
  const isDecimal = definition.valueType === "decimal";

  return (
    <div className="pe-parameter-row">
      <ParameterInfo definition={definition} descriptionId={descriptionId} t={t} />
      <div className="pe-value-control">
        <input
          type={isInteger || isDecimal ? "number" : "text"}
          step={isInteger ? "1" : isDecimal ? "any" : undefined}
          value={value}
          onChange={(event) => onChange(event.target.value)}
          placeholder={t("ollama.parameterDefaultValue", { value: definition.defaultValue })}
          aria-label={definition.key}
          aria-describedby={descriptionId}
          maxLength={MAX_PARAMETER_VALUE_LENGTH}
          className="pe-input pe-fixed-input"
        />
        {value && (
          <Tooltip label={t("ollama.useDefaultValue")}>
            <button type="button" className="ollama-btn pe-clear-btn" onClick={() => onChange("")}>
              ×
            </button>
          </Tooltip>
        )}
      </div>
    </div>
  );
}

interface StopParameterFieldProps {
  definition: ModelParameterDefinition;
  values: string[];
  rowIds: string[];
  t: TFunction;
  onChange: (index: number, value: string) => void;
  onAdd: () => void;
  onRemove: (index: number) => void;
}

export function StopParameterField({
  definition,
  values,
  rowIds,
  t,
  onChange,
  onAdd,
  onRemove,
}: StopParameterFieldProps) {
  const descriptionId = useId();

  return (
    <div className="pe-parameter-row pe-stop-row">
      <ParameterInfo definition={definition} descriptionId={descriptionId} t={t} />
      <div className="pe-stop-controls">
        {values.map((value, index) => (
          <div className="pe-value-control" key={rowIds[index]}>
            <input
              value={value}
              onChange={(event) => onChange(index, event.target.value)}
              placeholder={t("ollama.stopSequencePlaceholder")}
              aria-label={`${definition.key} ${index + 1}`}
              aria-describedby={descriptionId}
              maxLength={MAX_PARAMETER_VALUE_LENGTH}
              className="pe-input pe-fixed-input"
            />
            <Tooltip label={t("ollama.removeStopSequence")}>
              <button
                type="button"
                className="ollama-btn pe-clear-btn"
                aria-label={t("ollama.removeStopSequence")}
                onClick={() => onRemove(index)}
              >
                ×
              </button>
            </Tooltip>
          </div>
        ))}
        <button
          type="button"
          className="ollama-btn pe-inline-add-btn"
          onClick={onAdd}
          disabled={values.length >= MAX_STOP_SEQUENCES}
        >
          {t("ollama.addStopSequence")}
        </button>
      </div>
    </div>
  );
}

function ParameterInfo({ definition, descriptionId, t }: {
  definition: ModelParameterDefinition;
  descriptionId: string;
  t: TFunction;
}) {
  return (
    <div className="pe-parameter-info">
      <div className="pe-parameter-heading">
        <code className="pe-parameter-name">{definition.key}</code>
        <span className="pe-type-badge">
          {t(`ollama.parameterTypes.${definition.valueType}`)}
        </span>
        {definition.defaultValue !== null && (
          <span className="pe-default-badge">
            {t("ollama.parameterDefaultShort", { value: definition.defaultValue })}
          </span>
        )}
      </div>
      <p id={descriptionId} className="pe-parameter-description">
        {t(`ollama.parameterDescriptions.${definition.key}`)}
      </p>
    </div>
  );
}
