import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { CustomParameterFields } from "./custom-parameter-fields";
import { ModelEditorShell } from "./model-editor-shell";
import {
  MODEL_PARAMETER_DEFINITIONS,
  MODEL_PARAMETER_GROUPS,
  type SingleValueParameterKey,
} from "./model-parameter-catalog";
import {
  buildParameterPayload,
  createParameterRowId,
  createParameterEditorState,
  hasInvalidCustomParameter,
  hasInvalidOfficialParameter,
  type ParameterEditorState,
} from "./parameter-editor-state";
import { ParameterField, StopParameterField } from "./parameter-fields";
import type { ModelParameter } from "./modelfile-utils";
import "./ollama.css";
import "./parameters-editor.css";

interface ParametersEditorProps {
  modelName: string;
  initialParameters: ModelParameter[];
  onSave: () => void;
  onCancel: () => void;
}

export function ParametersEditor({
  modelName, initialParameters, onSave, onCancel,
}: ParametersEditorProps) {
  const { t } = useTranslation();
  const [editorState, setEditorState] = useState<ParameterEditorState>(
    () => createParameterEditorState(initialParameters),
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const updateValue = (key: SingleValueParameterKey, value: string) => {
    setEditorState((current) => ({
      ...current,
      values: { ...current.values, [key]: value },
    }));
  };

  const updateStop = (index: number, value: string) => {
    setEditorState((current) => ({
      ...current,
      stopValues: current.stopValues.map((entry, itemIndex) => (
        itemIndex === index ? value : entry
      )),
    }));
  };

  const removeStop = (index: number) => {
    setEditorState((current) => {
      const stopValues = current.stopValues.filter((_, itemIndex) => itemIndex !== index);
      const stopIds = current.stopIds.filter((_, itemIndex) => itemIndex !== index);
      if (stopValues.length > 0) return { ...current, stopValues, stopIds };
      return { ...current, stopValues: [""], stopIds: [createParameterRowId()] };
    });
  };

  const updateCustom = (index: number, field: "key" | "value", value: string) => {
    setEditorState((current) => ({
      ...current,
      customParameters: current.customParameters.map((parameter, itemIndex) => (
        itemIndex === index ? { ...parameter, [field]: value } : parameter
      )),
    }));
  };

  const removeCustom = (index: number) => {
    setEditorState((current) => ({
      ...current,
      customParameters: current.customParameters.filter((_, itemIndex) => itemIndex !== index),
      customParameterIds: current.customParameterIds.filter(
        (_, itemIndex) => itemIndex !== index,
      ),
    }));
  };

  const handleSave = async () => {
    setSaving(true);
    setError(null);
    try {
      if (hasInvalidCustomParameter(editorState)) {
        setError(t("ollama.invalidCustomParameter"));
        return;
      }
      if (hasInvalidOfficialParameter(editorState)) {
        setError(t("ollama.invalidOfficialParameter"));
        return;
      }
      const payload = buildParameterPayload(editorState);
      await invoke("update_parameters", { name: modelName, parameters: payload });
      onSave();
    } catch {
      setError(t("errors.operationFailed"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <ModelEditorShell
      title={`${modelName} — ${t("ollama.parameters")}`}
      cancelLabel={t("ollama.cancel")}
      saveLabel={t("ollama.save")}
      saving={saving}
      error={error}
      onCancel={onCancel}
      onSave={() => void handleSave()}
    >
      <div className="pe-body">
        <p className="pe-intro">{t("ollama.parameterEditorHint")}</p>

        {MODEL_PARAMETER_GROUPS.map((group) => (
          <section className="pe-group" key={group}>
            <h3 className="pe-group-title">{t(`ollama.parameterGroups.${group}`)}</h3>
            {MODEL_PARAMETER_DEFINITIONS
              .filter((definition) => definition.group === group)
              .map((definition) => {
                if (definition.key === "stop") {
                  return (
                    <StopParameterField
                      key={definition.key}
                      definition={definition}
                      values={editorState.stopValues}
                      rowIds={editorState.stopIds}
                      t={t}
                      onChange={updateStop}
                      onAdd={() => setEditorState((current) => ({
                        ...current,
                        stopValues: [...current.stopValues, ""],
                        stopIds: [...current.stopIds, createParameterRowId()],
                      }))}
                      onRemove={removeStop}
                    />
                  );
                }
                const parameterKey = definition.key;
                return (
                  <ParameterField
                    key={parameterKey}
                    definition={definition}
                    value={editorState.values[parameterKey]}
                    t={t}
                    onChange={(value) => updateValue(parameterKey, value)}
                  />
                );
              })}
          </section>
        ))}

        <CustomParameterFields
          parameters={editorState.customParameters}
          rowIds={editorState.customParameterIds}
          t={t}
          onChange={updateCustom}
          onRemove={removeCustom}
          onAdd={() => setEditorState((current) => ({
            ...current,
            customParameters: [...current.customParameters, { key: "", value: "" }],
            customParameterIds: [...current.customParameterIds, createParameterRowId()],
          }))}
        />
      </div>
    </ModelEditorShell>
  );
}
