import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const tokensCss = readFileSync("src/styles/tokens.css", "utf8");
const chatCss = readFileSync("src/components/agent-local/chat.css", "utf8");

const messagesRule = /\.chat-messages\s*\{([^}]*)\}/.exec(chatCss)?.[1] ?? "";

describe("fondu de la zone de conversation", () => {
  it("tient sa hauteur d'un jeton partagé", () => {
    expect(tokensCss).toContain("--chat-fade-height:");
  });

  it("efface le contenu des deux côtés", () => {
    expect(messagesRule).toContain("black var(--chat-fade-height)");
    expect(messagesRule).toContain("black calc(100% - var(--chat-fade-height))");
  });

  it("garde la variante préfixée", () => {
    // L'application tourne dans le WebView de macOS : sans « -webkit- », le
    // masque est ignoré et la coupure redevient franche. Le défaut passerait
    // inaperçu en développement, où le navigateur accepte la forme standard.
    expect(messagesRule).toContain("-webkit-mask-image:");
    expect(messagesRule).toContain("mask-image:");
  });
});
