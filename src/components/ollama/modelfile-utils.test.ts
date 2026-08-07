import { describe, it, expect } from "vitest";
import { extractParameters } from "./modelfile-utils";

// --- extractParameters -----------------------------------------------------

describe("extractParameters", () => {
  it("retourne un tableau vide pour un modelfile vide", () => {
    expect(extractParameters("")).toEqual([]);
  });

  it("retourne un tableau vide pour un modelfile sans PARAMETER", () => {
    expect(extractParameters('FROM llama3\nSYSTEM "You are helpful"')).toEqual([]);
  });

  it("extrait un paramètre simple", () => {
    const modelfile = "FROM llama3\nPARAMETER temperature 0.7\n";
    const params = extractParameters(modelfile);

    expect(params).toHaveLength(1);
    expect(params[0]).toEqual({ key: "temperature", value: "0.7" });
  });

  it("extrait plusieurs paramètres", () => {
    const modelfile = [
      "FROM llama3",
      "PARAMETER temperature 0.8",
      "PARAMETER top_p 0.9",
      "PARAMETER stop <|im_start|>",
      "PARAMETER stop <|im_end|>",
    ].join("\n");
    const params = extractParameters(modelfile);

    expect(params).toHaveLength(4);
    expect(params[0].key).toBe("temperature");
    expect(params[1].key).toBe("top_p");
    expect(params[2].value).toBe("<|im_start|>");
    expect(params[3].value).toBe("<|im_end|>");
  });

  it("trim la valeur du paramètre", () => {
    const modelfile = "PARAMETER temperature   0.5   \n";
    const params = extractParameters(modelfile);

    expect(params[0].value).toBe("0.5");
  });

  it("gère un paramètre avec une valeur multi-mots", () => {
    const modelfile = "PARAMETER stop User:";
    const params = extractParameters(modelfile);

    expect(params[0].value).toBe("User:");
  });

  it("est insensible à la casse de PARAMETER", () => {
    const modelfile = "parameter temperature 0.5";
    expect(extractParameters(modelfile)).toEqual([{ key: "temperature", value: "0.5" }]);
  });

  it("borne le nombre de paramètres extraits", () => {
    const modelfile = Array.from(
      { length: 140 },
      (_, index) => `PARAMETER custom_${index} ${index}`,
    ).join("\n");

    expect(extractParameters(modelfile)).toHaveLength(128);
  });

  it("ignore le texte PARAMETER dans les blocs SYSTEM et TEMPLATE multilignes", () => {
    const modelfile = [
      "FROM llama3",
      'SYSTEM """',
      "PARAMETER num_ctx 99999 appartient au prompt",
      '"""',
      "PARAMETER temperature 0.5",
      'TEMPLATE """',
      "PARAMETER stop ne constitue pas une directive",
      '"""',
      "PARAMETER top_p 0.9",
    ].join("\n");

    expect(extractParameters(modelfile)).toEqual([
      { key: "temperature", value: "0.5" },
      { key: "top_p", value: "0.9" },
    ]);
  });
});
