import { describe, expect, it } from "vitest";
import errorContract from "./ollama-runtime-error-contract.json";
import { ollamaErrorKey, ollamaProgressKey } from "./ollama-runtime-error";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itJson from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";

describe("Ollama runtime error mapper", () => {
  it("allowlists every public code and returns only an i18n key", () => {
    for (const code of Object.keys(errorContract)) {
      expect(ollamaErrorKey(code)).toBe(errorContract[code as keyof typeof errorContract]);
    }
    expect(new Set(Object.values(errorContract)).size).toBe(Object.keys(errorContract).length);
  });

  it.each([
    undefined,
    null,
    42,
    {},
    "not-a-public-code",
    "/Users/secret/stack.trace",
    "x".repeat(257),
  ])("maps unsafe input %j to the generic key", (value) => {
    expect(ollamaErrorKey(value)).toBe("ollama.errors.generic");
  });

  it("maps progress through local keys and never returns backend text", () => {
    expect(ollamaProgressKey("downloading")).toBe("ollamaSetup.downloading");
    expect(ollamaProgressKey("backend says /tmp/secret")).toBe("ollama.errors.generic");
  });

  it.each([
    ["preparing", "ollamaSetup.preparing"],
    ["downloading", "ollamaSetup.downloading"],
    ["verifying", "ollamaSetup.verifying"],
    ["extracting", "ollamaSetup.extracting"],
    ["validating", "ollamaSetup.validating"],
    ["committing", "ollamaSetup.committing"],
    ["starting", "ollamaSetup.starting"],
    ["recovering", "ollamaSetup.recovering"],
    ["rolling_back", "ollamaSetup.rollingBack"],
    ["cleaning", "ollamaSetup.cleaning"],
  ])("maps progress stage %s to its exact label", (stage, key) => {
    expect(ollamaProgressKey(stage)).toBe(key);
  });

  it("provides every Rust-owned error key in all seven locales", () => {
    const locales: Record<string, { ollama: { errors: Record<string, string> } }> = {
      de, en, es, fr, it: itJson, ja, zh,
    };
    for (const [language, locale] of Object.entries(locales)) {
      for (const key of Object.values(errorContract)) {
        const leaf = key.slice("ollama.errors.".length);
        expect(locale.ollama.errors[leaf], `${language}:${key}`).toBeTypeOf("string");
        expect(locale.ollama.errors[leaf].trim(), `${language}:${key}`).not.toBe("");
      }
    }
  });

  it("provides every progress key in all seven locales", () => {
    const locales = { de, en, es, fr, it: itJson, ja, zh };
    const keys = [
      "preparing", "downloading", "verifying", "extracting", "validating",
      "committing", "starting", "recovering", "rollingBack", "cleaning",
    ] as const;
    for (const [language, locale] of Object.entries(locales)) {
      const setup = locale.ollamaSetup as Record<string, string | undefined>;
      for (const key of keys) {
        const value = setup[key];
        expect(value, `${language}:ollamaSetup.${key}`).toBeTypeOf("string");
        expect(value?.trim(), `${language}:ollamaSetup.${key}`).not.toBe("");
      }
    }
  });
});
