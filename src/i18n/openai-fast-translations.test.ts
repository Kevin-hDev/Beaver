import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import itMessages from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

const translations = [
  ["fr", fr, "Rapide"],
  ["en", en, "Fast"],
  ["es", es, "Rápido"],
  ["de", de, "Schnell"],
  ["it", itMessages, "Rapido"],
  ["zh", zh, "快速"],
  ["ja", ja, "高速"],
] as const;

interface FastTranslation {
  agentLocal: { fastMode?: string };
  errors: { serviceTierUnavailable?: string; sessionSaveFailed?: string };
}

describe("traductions OpenAI Fast", () => {
  it.each(translations)("fournit le libellé exact et les erreurs en %s", (_locale, messages, label) => {
    const typedMessages = messages as unknown as FastTranslation;
    expect(typedMessages.agentLocal.fastMode).toBe(label);
    expect(typedMessages.errors.serviceTierUnavailable).toBeTruthy();
    expect(typedMessages.errors.sessionSaveFailed).toBeTruthy();
  });
});
