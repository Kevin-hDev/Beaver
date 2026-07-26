import { describe, expect, it } from "vitest";
import {
  MAX_CUSTOM_PARAMETERS,
  MAX_STOP_SEQUENCES,
  buildParameterPayload,
  createParameterEditorState,
  hasInvalidCustomParameter,
  hasInvalidOfficialParameter,
} from "./parameter-editor-state";

describe("parameter editor state", () => {
  it("borne les stops et les paramètres personnalisés provenant du Modelfile", () => {
    const initial = [
      ...Array.from({ length: 40 }, (_, index) => ({ key: "stop", value: `stop-${index}` })),
      ...Array.from({ length: 80 }, (_, index) => ({ key: `custom_${index}`, value: `${index}` })),
    ];

    const state = createParameterEditorState(initial);

    expect(state.stopValues).toHaveLength(MAX_STOP_SEQUENCES);
    expect(state.customParameters).toHaveLength(MAX_CUSTOM_PARAMETERS);
  });

  it("normalise les clés officielles et conserve les clés personnalisées", () => {
    const state = createParameterEditorState([
      { key: "TEMPERATURE", value: "0.5" },
      { key: "future_option", value: "enabled" },
    ]);

    expect(state.values.temperature).toBe("0.5");
    expect(state.customParameters).toEqual([{ key: "future_option", value: "enabled" }]);
  });

  it("retire les valeurs vides du payload", () => {
    const state = createParameterEditorState([
      { key: "num_ctx", value: " 32768 " },
      { key: "stop", value: "" },
      { key: "future_option", value: " yes " },
    ]);

    expect(buildParameterPayload(state)).toEqual([
      ["num_ctx", "32768"],
      ["future_option", "yes"],
    ]);
  });

  it("refuse une clé personnalisée invalide ou déjà officielle", () => {
    const reserved = createParameterEditorState([{ key: "future_option", value: "1" }]);
    reserved.customParameters[0].key = "temperature";
    expect(hasInvalidCustomParameter(reserved)).toBe(true);

    reserved.customParameters[0].key = "invalid-key";
    expect(hasInvalidCustomParameter(reserved)).toBe(true);
  });

  it("refuse les entiers non stricts et les décimaux non finis", () => {
    const state = createParameterEditorState([]);
    state.values.num_ctx = "1.5";
    expect(hasInvalidOfficialParameter(state)).toBe(true);

    state.values.num_ctx = "32768";
    state.values.temperature = "1e309";
    expect(hasInvalidOfficialParameter(state)).toBe(true);

    state.values.temperature = "0.7";
    expect(hasInvalidOfficialParameter(state)).toBe(false);
  });
});
