import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { BrowserTabState } from "../browser-types";
import { BrowserTabStrip } from "../browser-tab-strip";

const TAB_ONE = "11111111111111111111111111111111";
const TAB_TWO = "22222222222222222222222222222222";
const TAB_THREE = "33333333333333333333333333333333";
const platform = vi.hoisted(() => ({ alt: "Alt" }));

vi.mock("@/lib/platform", () => ({
  get ALT_LABEL() { return platform.alt; },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: { title?: string; alt?: string }) => ({
      "browser.tabsLabel": "Onglets du navigateur",
      "browser.reorderTabsHint": `Déplacez avec ${values?.alt} + Shift + Left / Right.`,
      "browser.newTab": "Nouvel onglet",
      "browser.closeTab": `Fermer ${values?.title ?? ""}`,
      "browser.addTab": "Ajouter un onglet",
    }[key] ?? key),
  }),
}));

function tabs(): BrowserTabState[] {
  return [
    { id: TAB_ONE, title: "A", url: "https://a.example/", loading: false, canGoBack: false, canGoForward: false, released: false },
    { id: TAB_TWO, title: "B", url: "https://b.example/", loading: false, canGoBack: false, canGoForward: false, released: false },
    { id: TAB_THREE, title: "C", url: "https://c.example/", loading: false, canGoBack: false, canGoForward: false, released: false },
  ];
}

function renderStrip(overrides: Partial<React.ComponentProps<typeof BrowserTabStrip>> = {}) {
  const props = {
    tabs: tabs(),
    activeTabId: TAB_ONE,
    enabled: true,
    conversationId: "conversation-a",
    onReorder: vi.fn().mockResolvedValue(true),
    onReorderError: vi.fn(),
    onSelect: vi.fn(),
    onClose: vi.fn(),
    onAdd: vi.fn(),
    ...overrides,
  };
  const parentPointerDown = vi.fn();
  const view = render(
    <div onPointerDown={parentPointerDown}>
      <BrowserTabStrip {...props} />
    </div>,
  );
  return { ...view, props, parentPointerDown };
}

beforeEach(() => {
  platform.alt = "Alt";
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
    this: HTMLElement,
  ) {
    const index = [TAB_ONE, TAB_TWO, TAB_THREE].indexOf(this.getAttribute("data-drag-id") ?? "");
    if (index >= 0) return { left: index * 100, top: 0, width: 100, height: 32 } as DOMRect;
    return { left: 0, top: 0, width: 300, height: 32 } as DOMRect;
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  cleanup();
});

describe("BrowserTabStrip", () => {
  it("déplace A après C sans sélectionner l'onglet au clic suivant", async () => {
    const { props, parentPointerDown } = renderStrip();
    const tab = screen.getByRole("tab", { name: "A" });

    fireEvent.pointerDown(tab, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 250, clientY: 10 });
    fireEvent.pointerUp(window);
    fireEvent.click(tab, { detail: 1 });

    await waitFor(() => expect(props.onReorder).toHaveBeenCalledWith([TAB_TWO, TAB_THREE, TAB_ONE]));
    expect(props.onSelect).not.toHaveBeenCalled();
    expect(parentPointerDown).not.toHaveBeenCalled();
  });

  it("permet Enter et Space après un glissement sans sélectionner au relâchement", async () => {
    const user = userEvent.setup();
    const { props } = renderStrip();
    const tab = screen.getByRole("tab", { name: "A" });
    fireEvent.pointerDown(tab, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 250, clientY: 10 });
    fireEvent.pointerUp(window);
    fireEvent.click(tab, { detail: 1 });
    expect(props.onSelect).not.toHaveBeenCalled();
    screen.getByRole("tab", { name: "B" }).focus();
    await user.keyboard("{Enter}");
    expect(props.onSelect).toHaveBeenLastCalledWith(TAB_TWO);
    screen.getByRole("tab", { name: "C" }).focus();
    await user.keyboard(" ");
    expect(props.onSelect).toHaveBeenLastCalledWith(TAB_THREE);
  });

  it("atteint la première place depuis B ou C", async () => {
    const { props } = renderStrip();
    const middle = screen.getByRole("tab", { name: "B" });

    fireEvent.pointerDown(middle, { button: 0, clientX: 110, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 5, clientY: 10 });
    fireEvent.pointerUp(window);
    await waitFor(() => expect(props.onReorder).toHaveBeenCalledWith([TAB_TWO, TAB_ONE, TAB_THREE]));
  });

  it("conserve le clic simple et laisse la croix hors de la poignée", () => {
    const { props } = renderStrip();

    fireEvent.click(screen.getByRole("tab", { name: "B" }));
    fireEvent.click(screen.getByRole("button", { name: "Fermer C" }));

    expect(props.onSelect).toHaveBeenCalledWith(TAB_TWO);
    expect(props.onClose).toHaveBeenCalledWith(TAB_THREE);
  });

  it("distingue les onglets qui portent le même titre par leur ID", () => {
    const duplicated = tabs().map((tab) => ({ ...tab, title: "Même titre" }));
    const { props } = renderStrip({ tabs: duplicated });

    fireEvent.click(screen.getAllByRole("tab", { name: "Même titre" })[1]);
    expect(props.onSelect).toHaveBeenCalledWith(TAB_TWO);
  });

  it("annule Échap et la fermeture d'onglet pendant le geste", () => {
    const { props, rerender } = renderStrip();
    const tab = screen.getByRole("tab", { name: "A" });

    fireEvent.pointerDown(tab, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 150, clientY: 10 });
    fireEvent.keyDown(window, { key: "Escape" });
    fireEvent.pointerUp(window);
    expect(props.onReorder).not.toHaveBeenCalled();

    fireEvent.pointerDown(tab, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 150, clientY: 10 });
    rerender(
      <BrowserTabStrip {...props} tabs={props.tabs.slice(1)} activeTabId={TAB_TWO} />,
    );
    fireEvent.pointerUp(window);
    expect(props.onReorder).not.toHaveBeenCalled();
  });

  it("annonce l'aide interpolée avec les libellés Alt et ⌥", () => {
    const first = renderStrip();
    const group = screen.getByRole("tablist", { name: "Onglets du navigateur" });
    expect(group).toHaveAttribute("aria-describedby");
    expect(document.getElementById(group.getAttribute("aria-describedby") ?? "")).toHaveTextContent("Alt");
    first.unmount();

    platform.alt = "⌥";
    renderStrip();
    expect(screen.getByText("Déplacez avec ⌥ + Shift + Left / Right.")).toHaveClass("sr-only");
  });

  it("réordonne au clavier, sans appel aux extrémités", async () => {
    const { props } = renderStrip();
    const middle = screen.getByRole("tab", { name: "B" });

    fireEvent.keyDown(middle, { key: "ArrowLeft", altKey: true, shiftKey: true });
    await waitFor(() => expect(props.onReorder).toHaveBeenCalledWith([TAB_TWO, TAB_ONE, TAB_THREE]));
    fireEvent.keyDown(screen.getByRole("tab", { name: "A" }), { key: "ArrowLeft", altKey: true, shiftKey: true });
    expect(props.onReorder).toHaveBeenCalledTimes(1);
  });
});
