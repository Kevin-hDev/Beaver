import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { resolveTerminalFont, TERMINAL_FONT_SIZE } from "../terminal-font";

/* xterm mesure la cellule une seule fois, à l'ouverture, et ne remesure qu'au
   changement de famille. Mesurée sur la police de secours pendant que la police
   web arrive, la cellule reste trop courte et le bas des lettres qui descendent
   est coupé. On ne nomme donc que la police réellement rendue, et on renomme la
   pile complète quand elle devient utilisable. */

const STACK = "'Beaver Mono Variable', ui-monospace, monospace";
const FALLBACK = "ui-monospace, monospace";

const doubles = vi.hoisted(() => ({ stack: "" }));

vi.mock("../terminal-theme", () => ({ readTerminalFont: () => doubles.stack }));

function stubFontSet(loaded: boolean, load = vi.fn()) {
  Object.defineProperty(document, "fonts", {
    configurable: true,
    value: { check: vi.fn(() => loaded), load },
  });
}

beforeEach(() => {
  doubles.stack = STACK;
});

afterEach(() => {
  Reflect.deleteProperty(document, "fonts");
});

describe("police de l'écran du terminal", () => {
  it("nomme la pile complète tout de suite quand la police est déjà là", () => {
    stubFontSet(true);

    const font = resolveTerminalFont();

    expect(font.family).toBe(STACK);
    expect(font.completed).toBeNull();
  });

  it("ne nomme que la police de secours tant que la police web manque", () => {
    stubFontSet(false);

    const font = resolveTerminalFont();

    expect(font.family).toBe(FALLBACK);
    expect(font.completed).not.toBeNull();
  });

  it("nomme la pile complète dès que la police web devient utilisable", async () => {
    const load = vi.fn(() => Promise.resolve([]));
    stubFontSet(false, load);

    const font = resolveTerminalFont();

    await expect(font.completed).resolves.toBe(STACK);
    expect(load).toHaveBeenCalledWith(`${TERMINAL_FONT_SIZE}px 'Beaver Mono Variable'`);
  });

  /* Sans autre famille à mesurer d'ici là, attendre n'apporterait rien. */
  it("n'attend pas quand la pile ne nomme aucune police de secours", () => {
    doubles.stack = "Beaver Mono";
    stubFontSet(false);

    const font = resolveTerminalFont();

    expect(font.family).toBe("Beaver Mono");
    expect(font.completed).toBeNull();
  });

  it("n'attend pas quand le moteur n'expose pas ses polices", () => {
    Reflect.deleteProperty(document, "fonts");

    const font = resolveTerminalFont();

    expect(font.family).toBe(STACK);
    expect(font.completed).toBeNull();
  });

  /* La police n'arrivera pas : la pile de secours est déjà celle qui est rendue
     et elle a été mesurée. Rien à reprendre, donc rien à résoudre. */
  it("garde la police de secours quand le chargement échoue", async () => {
    const load = vi.fn(() => Promise.reject(new Error("réseau")));
    stubFontSet(false, load);

    const font = resolveTerminalFont();
    const renomme = vi.fn();
    void font.completed?.then(renomme);
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(font.family).toBe(FALLBACK);
    expect(renomme).not.toHaveBeenCalled();
  });
});
