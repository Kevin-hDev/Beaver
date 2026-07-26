import { describe, expect, it, beforeEach } from "vitest";
import {
  resetWebToolToastDedupeForTest,
  sanitizeWebToolError,
  webToolErrorToastMessage,
} from "../web-tool-error-toast";
import type { StreamEvent } from "@/types/agent";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itJson from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";

describe("webToolErrorToastMessage", () => {
  beforeEach(() => resetWebToolToastDedupeForTest());

  it("affiche un toast pour une erreur web_search finale", () => {
    const msg = webToolErrorToastMessage("s1", toolError("web_search", 2, "Brave: HTTP 429"));
    expect(msg).toContain("Brave: HTTP 429");
  });

  it("ne duplique pas le même toolCallIndex", () => {
    const event = toolError("web_search", 4, "SearXNG: timeout au démarrage");
    expect(webToolErrorToastMessage("s1", event)).not.toBeNull();
    expect(webToolErrorToastMessage("s1", event)).toBeNull();
  });

  it("ignore les erreurs web_fetch côté UI", () => {
    expect(webToolErrorToastMessage("s1", toolError("web_fetch", 4, "HTTP 403"))).toBeNull();
    expect(
      webToolErrorToastMessage("s1", toolError("web_fetch", 5, "Erreur fetch: requête échouée"))
    ).toBeNull();
  });

  it("garde les erreurs provider web_search visibles", () => {
    const msg = webToolErrorToastMessage("s1", toolError("web_search", 2, "Brave: HTTP 429"));
    expect(msg).toContain("Brave: HTTP 429");
  });

  it("ignore les fallbacks réussis et les outils non web", () => {
    expect(webToolErrorToastMessage("s1", toolOk("web_search", 1))).toBeNull();
    expect(webToolErrorToastMessage("s1", toolError("grep", 1, "bad"))).toBeNull();
  });
});

describe("sanitizeWebToolError", () => {
  it("retire secrets et chemins locaux du détail visible", () => {
    const msg = sanitizeWebToolError("Bearer abcdefghijkl secret_key=abc123456 /Users/me/app/file.ts");
    expect(msg).toContain("Bearer [redacted]");
    expect(msg).toContain("secret_key=[redacted]");
    expect(msg).toContain("[path]");
  });
});

describe("i18n", () => {
  it("contient la clé webToolFailed dans les 7 langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as Array<{ errors: { webToolFailed: string } }>;
    for (const locale of locales) {
      expect(locale.errors.webToolFailed).toContain("{{message}}");
    }
  });
});

function toolError(name: string, toolCallIndex: number, content: string): StreamEvent {
  return { event: "toolResult", data: { name, content, isError: true, toolCallIndex } };
}

function toolOk(name: string, toolCallIndex: number): StreamEvent {
  return { event: "toolResult", data: { name, content: "ok", isError: false, toolCallIndex } };
}
