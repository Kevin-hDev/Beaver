import { describe, it, expect } from "vitest";
import { extractParameters, extractSystemPrompt } from "./modelfile-utils";

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
});

// --- extractSystemPrompt ---------------------------------------------------

describe("extractSystemPrompt", () => {
  it("retourne une chaîne vide pour un modelfile vide", () => {
    expect(extractSystemPrompt("")).toBe("");
  });

  it("retourne une chaîne vide si pas de SYSTEM", () => {
    expect(extractSystemPrompt("FROM llama3\nPARAMETER temperature 0.7")).toBe("");
  });

  it("extrait un SYSTEM en triple quotes", () => {
    const modelfile = 'FROM llama3\nSYSTEM """\nYou are a helpful assistant.\nStay concise.\n"""';
    const prompt = extractSystemPrompt(modelfile);

    expect(prompt).toContain("You are a helpful assistant.");
    expect(prompt).toContain("Stay concise.");
  });

  it("extrait un SYSTEM en simple quote", () => {
    const modelfile = 'FROM llama3\nSYSTEM "You are helpful"';
    const prompt = extractSystemPrompt(modelfile);

    expect(prompt).toBe("You are helpful");
  });

  it("extrait un SYSTEM sans quote (bare)", () => {
    const modelfile = "FROM llama3\nSYSTEM You are helpful";
    const prompt = extractSystemPrompt(modelfile);

    expect(prompt).toBe("You are helpful");
  });

  it("priorise les triple quotes sur les autres formats", () => {
    const modelfile = 'SYSTEM """\nTriple quote content\n"""';
    const prompt = extractSystemPrompt(modelfile);

    expect(prompt).toBe("Triple quote content");
  });

  it("trim le contenu des triple quotes", () => {
    const modelfile = 'SYSTEM """\n\n  Content with spaces  \n\n"""';
    const prompt = extractSystemPrompt(modelfile);

    expect(prompt).toBe("Content with spaces");
  });

  it("gère un SYSTEM sur une seule ligne en simple quote sans \n interne", () => {
    // La regex simple quote exclut les \n dans le match.
    const modelfile = 'SYSTEM "Line one"';
    const prompt = extractSystemPrompt(modelfile);

    expect(prompt).toBe("Line one");
  });
});
