import { describe, expect, it } from "vitest";
import de from "./de.json";
import en from "./en.json";
import es from "./es.json";
import fr from "./fr.json";
import italian from "./it.json";
import ja from "./ja.json";
import zh from "./zh.json";

type JsonObject = Record<string, unknown>;

function compressionKeys(value: unknown, path = ""): string[] {
  if (!value || typeof value !== "object") return [];
  return Object.entries(value as JsonObject).flatMap(([key, child]) => {
    const next = path ? `${path}.${key}` : key;
    if (child && typeof child === "object") return compressionKeys(child, next);
    return next.split(".").some((part) => part.toLowerCase().startsWith("compression"))
      ? [next]
      : [];
  });
}

const SOURCES: Record<string, string> = import.meta.glob(
  "/src/components/{settings/compression,agent-local}/*.tsx",
  { eager: true, query: "?raw", import: "default" },
);

describe("compression profile translations", () => {
  it("garde toutes les surfaces de compression identiques dans les sept langues", () => {
    const expected = compressionKeys(en).sort();
    for (const locale of [fr, es, de, italian, zh, ja]) {
      expect(compressionKeys(locale).sort()).toEqual(expected);
    }
  });

  it("ne laisse aucun texte visible codé en dur dans les nouveaux composants", () => {
    const agentFiles = new Set([
      "/src/components/agent-local/chat-plus-compression-menu.tsx",
      "/src/components/agent-local/context-compression-help-popover.tsx",
      "/src/components/agent-local/context-progress.tsx",
    ]);
    const violations = Object.entries(SOURCES).flatMap(([file, source]) => {
      if (file.includes("/agent-local/") && !agentFiles.has(file)) return [];
      const textNodes = [...source.matchAll(/<(?:button|div|h[1-6]|p|span|strong)[^>]*>\s*([A-Za-zÀ-ÿ][^<{>\n]*)\s*</g)]
        .map((match) => match[1].trim());
      const attributes = [...source.matchAll(/(?:aria-label|placeholder|title)="([A-Za-zÀ-ÿ][^"]*)"/g)]
        .map((match) => match[1]);
      return [...textNodes, ...attributes].map((text) => `${file}: ${text}`);
    });

    expect(violations).toEqual([]);
  });
});
