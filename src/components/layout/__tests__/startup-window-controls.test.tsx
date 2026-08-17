import { readFileSync } from "node:fs";
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { StartupWindowControls } from "../startup-window-controls";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "fr", changeLanguage: vi.fn() },
  }),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    startDragging: vi.fn().mockResolvedValue(undefined),
    isMaximized: vi.fn().mockResolvedValue(false),
    maximize: vi.fn().mockResolvedValue(undefined),
    unmaximize: vi.fn().mockResolvedValue(undefined),
    minimize: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  }),
}));

function stackingOrder(source: string, selector: string): number {
  const start = source.indexOf(selector);
  const rule = source.slice(start, source.indexOf("}", start));
  const found = /z-index:\s*(\d+)/.exec(rule);
  if (!found) throw new Error(`aucun z-index dans la règle ${selector}`);
  return Number(found[1]);
}

afterEach(cleanup);

describe("boutons de fenêtre des écrans de démarrage", () => {
  it("reprend la primitive de l'application plutôt qu'un second jeu de boutons", () => {
    const { container } = render(<StartupWindowControls />);

    expect(container.querySelector(".window-controls")).not.toBeNull();
    expect(container.querySelectorAll(".wc-btn")).toHaveLength(3);
  });

  /* Le splash est un calque posé dans index.html, hors de l'arbre React et
     au-dessus de tout. Sans ce cran d'écart, les boutons existent mais restent
     cachés derrière lui pendant tout le chargement. */
  it("passe au-dessus du calque du splash", () => {
    const splash = stackingOrder(readFileSync("index.html", "utf8"), "#splash {");
    const controls = stackingOrder(
      readFileSync("src/components/layout/startup-window-controls.css", "utf8"),
      ".startup-window-controls {",
    );

    expect(controls).toBeGreaterThan(splash);
  });

  /* Le conteneur ne doit rien peser dans la mise en page : il est inséré dans
     des coquilles en flux (l'accueil est une colonne flex, l'installation
     d'Ollama une boîte centrée) et n'a que des enfants en position fixe. */
  it("ne prend aucune place dans le flux", () => {
    const css = readFileSync("src/components/layout/startup-window-controls.css", "utf8");
    const start = css.indexOf(".startup-window-controls {");
    const rule = css.slice(start, css.indexOf("}", start));

    expect(rule).toContain("position: relative;");
    expect(rule).not.toMatch(/\b(height|width|padding|margin|flex):/);
  });
});
