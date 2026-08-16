import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TerminalTabBar } from "../terminal-tab-bar";
import type { TerminalTab } from "@/hooks/use-terminal";

function tab(id: string, extra: Partial<TerminalTab> = {}): TerminalTab {
  return { id, ptyId: null, ptyToken: null, label: id, cwd: "/tmp", hasActivity: false, ...extra };
}

function renderBar(tabs: TerminalTab[], activeTabId: string | null, handlers = {}) {
  const props = {
    onSelect: vi.fn(), onClose: vi.fn(), onAdd: vi.fn(),
    onRename: vi.fn(), onReorder: vi.fn(), onClosePanel: vi.fn(),
    ...handlers,
  };
  render(<TerminalTabBar tabs={tabs} activeTabId={activeTabId} {...props} />);
  return props;
}

function bar(): HTMLElement {
  return document.querySelector(".terminal-tab-bar") as HTMLElement;
}

describe("barre d'onglets du terminal", () => {
  /* La croix vivait sous l'icône, à gauche : survoler l'onglet pour l'ouvrir
     la faisait apparaître sous le curseur, et viser l'icône fermait l'onglet. */
  it("donne à chaque onglet sa croix, posée après le libellé", () => {
    renderBar([tab("build"), tab("serveur")], "build");

    const items = document.querySelectorAll(".terminal-tab-item");
    expect(items).toHaveLength(2);
    for (const item of items) {
      const parts = [...item.children].map((child) => child.className);
      expect(parts.indexOf("terminal-tab-close")).toBe(parts.length - 1);
    }
  });

  it("ferme l'onglet visé sans l'ouvrir au passage", () => {
    const props = renderBar([tab("build"), tab("serveur")], "build");

    const close = document.querySelectorAll(".terminal-tab-close")[1];
    fireEvent.click(close);

    expect(props.onClose).toHaveBeenCalledWith("serveur");
    expect(props.onSelect).not.toHaveBeenCalled();
  });

  it("ouvre l'onglet qu'on désigne", () => {
    const props = renderBar([tab("build"), tab("serveur")], "build");

    fireEvent.click(document.querySelectorAll(".terminal-tab-item")[1]);

    expect(props.onSelect).toHaveBeenCalledWith("serveur");
  });

  /* Le bouton portait une marge automatique, sans effet : l'élément que la
     rangée dispose est l'infobulle qui l'enveloppe, pas lui. */
  it("pose la fermeture du panneau en dernier, après l'espace qui la repousse", () => {
    renderBar([tab("build")], "build");

    const children = [...bar().children];
    const gap = children.findIndex((child) => child.className === "terminal-tab-gap");
    const closePanel = children.findIndex((child) => child.querySelector(".terminal-tab-bar-close"));

    expect(gap).toBeGreaterThan(-1);
    expect(closePanel).toBe(children.length - 1);
    expect(closePanel).toBeGreaterThan(gap);
  });

  /* Les boutons partageaient la zone qui défile et sortaient de l'écran dès
     qu'il y avait trop d'onglets. */
  it("garde les boutons hors de la zone qui défile", () => {
    renderBar([tab("build")], "build");

    const track = document.querySelector(".terminal-tab-track") as HTMLElement;
    expect(track.querySelector(".terminal-tab-add")).toBeNull();
    expect(track.querySelector(".terminal-tab-bar-close")).toBeNull();
    expect(track.querySelectorAll(".terminal-tab-item")).toHaveLength(1);
  });

  it("marque l'onglet laissé de côté qui a produit du texte", () => {
    renderBar([tab("build"), tab("serveur", { hasActivity: true })], "build");

    const items = document.querySelectorAll(".terminal-tab-item");
    expect(items[0].querySelector(".terminal-tab-dot")).toBeNull();
    expect(items[1].querySelector(".terminal-tab-dot")).not.toBeNull();
  });

  /* Sur l'onglet ouvert, la marque n'a rien à dire : on le regarde. */
  it("ne marque pas l'onglet ouvert", () => {
    renderBar([tab("build", { hasActivity: true })], "build");

    expect(document.querySelector(".terminal-tab-dot")).toBeNull();
  });

  it("ajoute un onglet et ferme le panneau par leurs boutons", () => {
    const props = renderBar([tab("build")], "build");

    fireEvent.click(document.querySelector(".terminal-tab-add") as HTMLElement);
    fireEvent.click(document.querySelector(".terminal-tab-bar-close") as HTMLElement);

    expect(props.onAdd).toHaveBeenCalled();
    expect(props.onClosePanel).toHaveBeenCalled();
  });

  it("renomme un onglet sur double-clic", () => {
    const props = renderBar([tab("build")], "build");

    fireEvent.doubleClick(document.querySelector(".terminal-tab-item") as HTMLElement);
    const input = screen.getByDisplayValue("build");
    fireEvent.change(input, { target: { value: "compilation" } });
    fireEvent.keyDown(input, { code: "Enter" });

    expect(props.onRename).toHaveBeenCalledWith("build", "compilation");
  });
});
