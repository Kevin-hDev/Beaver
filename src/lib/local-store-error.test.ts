import { describe, expect, it } from "vitest";
import { localStoreErrorMessage } from "./local-store-error";

const translate = (key: string) => key;

describe("localStoreErrorMessage", () => {
  it.each([
    ["ollama-custom-store-unavailable", "errors.localStore.ollamaCustomization"],
    ["ollama-native-prompt-store-unavailable", "errors.localStore.ollamaNativePrompts"],
    ["system-prompt-store-unavailable", "errors.localStore.systemPrompts"],
  ])("traduit %s sans exposer l’erreur brute", (error, expected) => {
    expect(localStoreErrorMessage(error, translate)).toBe(expected);
  });

  it("conserve une erreur générique pour une valeur inconnue", () => {
    expect(localStoreErrorMessage("/Users/private/data.json", translate))
      .toBe("errors.operationFailed");
  });
});
