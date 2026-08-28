import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import { describe, expect, it } from "vitest";
import { IS_MAC } from "@/lib/platform";

const indexHtml = readFileSync("index.html", "utf8");
const tokensCss = readFileSync("src/styles/tokens.css", "utf8");
const onboardingCss = readFileSync("src/components/onboarding/onboarding.css", "utf8");
const appCss = readFileSync("src/App.css", "utf8");
const bootstrapSource = indexHtml.match(/<script>([\s\S]*?)<\/script>/)?.[1];

function osFor(userAgent: string): string {
  const attributes: Record<string, string> = {};
  runInNewContext(bootstrapSource!, {
    localStorage: { getItem: () => "dark" },
    navigator: { userAgent },
    window: { matchMedia: () => ({ matches: true }) },
    document: {
      documentElement: {
        setAttribute: (name: string, value: string) => {
          attributes[name] = value;
        },
      },
    },
  });
  return attributes["data-os"];
}

describe("Coin de la fenêtre avant l'application", () => {
  it("nomme la plateforme dès l'amorçage, avant que React démarre", () => {
    expect(osFor("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")).toBe("mac");
    expect(osFor("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")).toBe("other");
    expect(osFor("Mozilla/5.0 (X11; Linux x86_64)")).toBe("other");
  });

  it("dit la même chose que src/lib/platform.ts", () => {
    expect(osFor(navigator.userAgent)).toBe(IS_MAC ? "mac" : "other");
  });

  it("arrondit le splash à la valeur du cadre de fenêtre", () => {
    // Le bloc du splash peint avant les feuilles de style : il ne peut pas lire
    // le jeton, il le recopie. Les deux doivent rester d'accord.
    const jeton = tokensCss.match(/--radius-window:\s*(\d+)px;/)?.[1];
    const splash = indexHtml.match(/\[data-os="other"\]\s*#splash\s*\{[^}]*border-radius:\s*(\d+)px;/)?.[1];
    expect(jeton).toBeDefined();
    expect(splash).toBe(jeton);
  });

  it("arrondit aussi l'onboarding et l'installation d'Ollama", () => {
    // Ces deux écrans occupent la fenêtre entière comme le splash : sans
    // décorations natives, c'est au contenu de porter le coin.
    expect(onboardingCss).toMatch(/\[data-os="other"\]\s*\.ob-shell\s*\{[^}]*border-radius:\s*var\(--radius-window\);/s);
    expect(appCss).toMatch(/\[data-os="other"\]\s*\.app-startup-shell\s*\{[^}]*border-radius:\s*var\(--radius-window\);/s);
  });

  it("laisse macOS dessiner son propre coin", () => {
    expect(indexHtml).not.toMatch(/\[data-os="mac"\]\s*#splash\s*\{[^}]*border-radius/);
    expect(onboardingCss).not.toMatch(/\[data-os="mac"\][^{]*\{[^}]*border-radius/);
  });
});
