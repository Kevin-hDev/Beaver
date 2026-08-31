import { describe, expect, it } from "vitest";
import { updateTab } from "../terminal-groups";
import type { TerminalGroup, TerminalTab } from "../terminal-types";

function tab(id: string, extra: Partial<TerminalTab> = {}): TerminalTab {
  return { id, ptyId: null, ptyToken: null, label: id, hasActivity: false, ...extra };
}

function groups(): Map<string, TerminalGroup> {
  return new Map([
    ["projet-a", { tabs: [tab("un"), tab("deux")], activeTabId: "un" }],
    ["projet-b", { tabs: [tab("trois")], activeTabId: "trois" }],
  ]);
}

describe("updateTab", () => {
  it("trouve l'onglet dans n'importe quel groupe", () => {
    const next = updateTab(groups(), "trois", { hasActivity: true });

    expect(next?.get("projet-b")?.tabs[0].hasActivity).toBe(true);
  });

  it("ne touche pas aux autres onglets du même groupe", () => {
    const next = updateTab(groups(), "deux", { label: "renommé" });

    expect(next?.get("projet-a")?.tabs[0].label).toBe("un");
    expect(next?.get("projet-a")?.tabs[1].label).toBe("renommé");
  });

  /* Sans ce retour, chaque ligne écrite par un programme bavard provoquerait un
     rendu — et une écriture sur le disque — pour reposer la valeur en place. */
  it("rend null quand la valeur est déjà celle demandée", () => {
    expect(updateTab(groups(), "un", { hasActivity: false })).toBeNull();
  });

  it("rend null quand l'onglet n'existe nulle part", () => {
    expect(updateTab(groups(), "absent", { hasActivity: true })).toBeNull();
  });

  it("laisse la carte d'origine intacte", () => {
    const before = groups();
    updateTab(before, "un", { hasActivity: true });

    expect(before.get("projet-a")?.tabs[0].hasActivity).toBe(false);
  });

  it("écrit plusieurs champs d'un coup", () => {
    const next = updateTab(groups(), "un", { ptyId: 42, ptyToken: "jeton" });
    const updated = next?.get("projet-a")?.tabs[0];

    expect(updated?.ptyId).toBe(42);
    expect(updated?.ptyToken).toBe("jeton");
  });

  /* Un seul champ qui change suffit à écrire, même si les autres sont en place. */
  it("écrit dès qu'un seul des champs demandés diffère", () => {
    const next = updateTab(groups(), "un", { hasActivity: false, ptyId: 7 });

    expect(next?.get("projet-a")?.tabs[0].ptyId).toBe(7);
  });
});
