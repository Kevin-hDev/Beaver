import { describe, expect, it } from "vitest";
import { updateTab } from "../terminal-groups";
import * as terminalGroups from "../terminal-groups";
import type { TerminalGroup, TerminalTab } from "../terminal-types";

type CloseTabInGroup = (
  groups: Map<string, TerminalGroup>,
  groupKey: string,
  tabId: string,
) => { groups: Map<string, TerminalGroup>; groupBecameEmpty: boolean; changed: boolean };

const closeTabInGroup = () => (
  terminalGroups as unknown as { closeTabInGroup?: CloseTabInGroup }
).closeTabInGroup;

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

describe("closeTabInGroup", () => {
  it("ferme l'onglet actif et sélectionne le suivant", () => {
    const close = closeTabInGroup();
    expect(close).toBeTypeOf("function");

    const result = close!(groups(), "projet-a", "un");

    expect(result.changed).toBe(true);
    expect(result.groupBecameEmpty).toBe(false);
    expect(result.groups.get("projet-a")?.tabs.map(({ id }) => id)).toEqual(["deux"]);
    expect(result.groups.get("projet-a")?.activeTabId).toBe("deux");
  });

  it("rend la carte intacte quand l'onglet est absent", () => {
    const before = groups();
    const result = closeTabInGroup()!(before, "projet-a", "absent");

    expect(result).toEqual({ groups: before, groupBecameEmpty: false, changed: false });
    expect(result.groups).toBe(before);
  });

  it("vide seulement le groupe explicitement ciblé", () => {
    const result = closeTabInGroup()!(groups(), "projet-b", "trois");

    expect(result.groupBecameEmpty).toBe(true);
    expect(result.groups.get("projet-b")).toEqual({ tabs: [], activeTabId: null });
    expect(result.groups.get("projet-a")?.tabs).toHaveLength(2);
  });
});
