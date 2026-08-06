import { describe, expect, it } from "vitest";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itJson from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";
import { localStoreErrorMessage } from "./local-store-error";

const translate = (key: string) => key;
const messageKeys = [
  "ollamaCustomization",
  "ollamaNativePrompts",
  "systemPrompts",
  "ollamaCustomizationMissing",
  "ollamaNativePromptsMissing",
  "systemPromptsMissing",
  "ollamaCustomizationWrite",
  "ollamaNativePromptsWrite",
  "systemPromptsWrite",
] as const;

type MessageKey = typeof messageKeys[number];
interface LocalStoreLocale {
  errors: { localStore: Record<MessageKey, string> };
}

describe("localStoreErrorMessage", () => {
  it.each([
    ["ollama-custom-store-unavailable", "errors.localStore.ollamaCustomization"],
    ["ollama-native-prompt-store-unavailable", "errors.localStore.ollamaNativePrompts"],
    ["system-prompt-store-unavailable", "errors.localStore.systemPrompts"],
    ["ollama-custom-store-missing", "errors.localStore.ollamaCustomizationMissing"],
    ["ollama-native-prompt-store-missing", "errors.localStore.ollamaNativePromptsMissing"],
    ["system-prompt-store-missing", "errors.localStore.systemPromptsMissing"],
    ["ollama-custom-store-write", "errors.localStore.ollamaCustomizationWrite"],
    ["ollama-native-prompt-write", "errors.localStore.ollamaNativePromptsWrite"],
    ["system-prompt-store-write", "errors.localStore.systemPromptsWrite"],
  ])("traduit %s sans exposer l’erreur brute", (error, expected) => {
    expect(localStoreErrorMessage(error, translate)).toBe(expected);
  });

  it("conserve une erreur générique pour une valeur inconnue", () => {
    expect(localStoreErrorMessage("/Users/private/data.json", translate))
      .toBe("errors.operationFailed");
  });

  it("fournit les messages de lecture, suppression et écriture dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as LocalStoreLocale[];
    for (const locale of locales) {
      for (const key of messageKeys) {
        expect(locale.errors.localStore[key].trim()).toBeTruthy();
      }
    }
  });
});
