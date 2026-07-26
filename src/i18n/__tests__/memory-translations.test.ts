import { describe, expect, it } from "vitest";
import de from "../de.json";
import en from "../en.json";
import es from "../es.json";
import fr from "../fr.json";
import italian from "../it.json";
import ja from "../ja.json";
import zh from "../zh.json";

interface MemoryLocale {
  settings: {
    tabs: { memory: string };
    memory: {
      intro: string;
      modes: { automatic: string };
      types: { preference: string };
      statuses: { archived: string };
      sources: { extractor: string };
      archive: string;
      confirmArchive: string;
      retry: string;
      tokenUnit: string;
      loadTopics: string;
      loadingTopics: string;
    };
  };
  agentLocal: {
    contextUsage: { categories: { memory: string } };
    toolActivity: { groups: { memory: string }; completed: string };
  };
  permissionDialog: Record<string, unknown>;
}

function memoryLocale(value: unknown): MemoryLocale {
  return value as MemoryLocale;
}

const locales = {
  de: memoryLocale(de),
  en: memoryLocale(en),
  es: memoryLocale(es),
  fr: memoryLocale(fr),
  it: memoryLocale(italian),
  ja: memoryLocale(ja),
  zh: memoryLocale(zh),
};

describe("traductions MEMORY", () => {
  it.each(Object.entries(locales))("%s expose toutes les sections visibles", (_locale, values) => {
    expect(values.settings.tabs.memory).toBeTruthy();
    expect(values.settings.memory.intro).toBeTruthy();
    expect(values.settings.memory.modes.automatic).toBeTruthy();
    expect(values.settings.memory.types.preference).toBeTruthy();
    expect(values.settings.memory.statuses.archived).toBeTruthy();
    expect(values.settings.memory.sources.extractor).toBeTruthy();
    expect(values.settings.memory.archive).toBeTruthy();
    expect(values.settings.memory.confirmArchive).toBeTruthy();
    expect(values.settings.memory.retry).toBeTruthy();
    expect(values.settings.memory.tokenUnit).toBeTruthy();
    expect(values.settings.memory.loadTopics).toBeTruthy();
    expect(values.settings.memory.loadingTopics).toBeTruthy();
    expect(values.agentLocal.contextUsage.categories.memory).toBeTruthy();
    expect(values.agentLocal.toolActivity.groups.memory).toBe("MEMORY");
    expect(values.agentLocal.toolActivity.completed).toBeTruthy();
    expect(values.permissionDialog).not.toHaveProperty("memory");
  });
});
