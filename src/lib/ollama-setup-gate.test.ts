import { describe, expect, it } from "vitest";
import {
  OLLAMA_SETUP_SKIPPED_KEY,
  hasSkippedOllamaSetup,
  ollamaSetupSkippedPatch,
  shouldShowOllamaSetup,
} from "./ollama-setup-gate";

describe("ollama setup gate", () => {
  it("affiche le setup quand Ollama manque et que le setup n'a pas ete passe", () => {
    expect(shouldShowOllamaSetup({ installed: false, skipped: false })).toBe(true);
  });

  it("n'affiche pas le setup quand Ollama est installe", () => {
    expect(shouldShowOllamaSetup({ installed: true, skipped: false })).toBe(false);
  });

  it("n'affiche pas le setup quand l'utilisateur l'a passe", () => {
    expect(shouldShowOllamaSetup({ installed: false, skipped: true })).toBe(false);
  });

  it("ne considere que la valeur booleenne true comme un skip valide", () => {
    expect(hasSkippedOllamaSetup({ [OLLAMA_SETUP_SKIPPED_KEY]: true })).toBe(true);
    expect(hasSkippedOllamaSetup({ [OLLAMA_SETUP_SKIPPED_KEY]: "true" })).toBe(false);
    expect(hasSkippedOllamaSetup(null)).toBe(false);
  });

  it("genere le patch de configuration attendu", () => {
    expect(ollamaSetupSkippedPatch(true)).toEqual({ [OLLAMA_SETUP_SKIPPED_KEY]: true });
    expect(ollamaSetupSkippedPatch(false)).toEqual({ [OLLAMA_SETUP_SKIPPED_KEY]: false });
  });
});
