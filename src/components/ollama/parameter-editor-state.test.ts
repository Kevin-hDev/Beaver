import { describe, expect, it } from "vitest";
import {
  MAX_CUSTOM_PARAMETERS,
  MAX_STOP_SEQUENCES,
  buildParameterPayload,
  createParameterEditorState,
  hasInvalidCustomParameter,
  hasInvalidOfficialParameter,
  hasUnsupportedParameterValue,
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

  it("retire seulement les valeurs exactement vides du payload", () => {
    const state = createParameterEditorState([
      { key: "num_ctx", value: " 32768 " },
      { key: "stop", value: "" },
      { key: "stop", value: " " },
      { key: "future_option", value: " yes " },
    ]);

    expect(buildParameterPayload(state)).toEqual([
      ["num_ctx", "32768"],
      ["stop", " "],
      ["future_option", " yes "],
    ]);
  });

  it("conserve les valeurs multilignes provenant du backend", () => {
    const state = createParameterEditorState([
      { key: "stop", value: "line one\nline two" },
      { key: "future_option", value: " first\nsecond " },
    ]);

    expect(buildParameterPayload(state)).toEqual([
      ["stop", "line one\nline two"],
      ["future_option", " first\nsecond "],
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

  it("refuse les retours chariot et les triples guillemets", () => {
    const state = createParameterEditorState([{ key: "stop", value: "safe" }]);
    state.stopValues[0] = "line one\rline two";
    expect(hasUnsupportedParameterValue(state)).toBe(true);

    state.stopValues[0] = 'a"""b';
    expect(hasUnsupportedParameterValue(state)).toBe(true);

    state.stopValues[0] = 'say "hi"\nnext';
    expect(hasUnsupportedParameterValue(state)).toBe(false);

    state.stopValues[0] = "unsafe\u0007value";
    expect(hasUnsupportedParameterValue(state)).toBe(true);
  });
});
