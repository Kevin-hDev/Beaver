import { fireEvent, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useSettings } from "@/hooks/use-settings";
import { useAppLayoutShortcuts } from "../use-app-layout-effects";

function renderShortcutsWithSettings() {
  renderHook(() => useAppLayoutShortcuts({
    onBack: vi.fn(),
    onForward: vi.fn(),
    onOpenSettings: vi.fn(),
    toggleSearch: vi.fn(),
    toggleSidebar: vi.fn(),
  }));
  return renderHook(() => useSettings());
}

describe("useAppLayoutShortcuts", () => {
  afterEach(() => {
    localStorage.clear();
    document.documentElement.style.fontSize = "";
  });

  it("ne capte pas Ctrl+Alt+B reserve a la preview", () => {
    const toggleSidebar = vi.fn();

    renderHook(() => useAppLayoutShortcuts({
      onBack: vi.fn(),
      onForward: vi.fn(),
      onOpenSettings: vi.fn(),
      toggleSearch: vi.fn(),
      toggleSidebar,
    }));

    fireEvent.keyDown(window, { code: "KeyB", ctrlKey: true, altKey: true });

    expect(toggleSidebar).not.toHaveBeenCalled();
  });

  it.each([
    ["Ctrl", { code: "Comma", key: ",", ctrlKey: true }],
    ["Cmd", { code: "Comma", key: ",", metaKey: true }],
  ])("ouvre les réglages avec %s + virgule", (_label, keyboard) => {
    const onOpenSettings = vi.fn();

    renderHook(() => useAppLayoutShortcuts({
      onBack: vi.fn(),
      onForward: vi.fn(),
      onOpenSettings,
      toggleSearch: vi.fn(),
      toggleSidebar: vi.fn(),
    }));

    fireEvent.keyDown(window, keyboard);

    expect(onOpenSettings).toHaveBeenCalledOnce();
  });

  it.each([
    ["Ctrl", { code: "Equal", key: "+", ctrlKey: true }],
    ["Cmd", { code: "Equal", key: "+", metaKey: true }],
    ["pavé numérique", { code: "NumpadAdd", key: "+", ctrlKey: true }],
  ])("agrandit l'interface avec %s +", (_label, keyboard) => {
    localStorage.setItem("clgo-font-size", "18");
    const settings = renderShortcutsWithSettings();

    fireEvent.keyDown(window, keyboard);

    expect(settings.result.current.fontSize).toBe(19);
    expect(document.documentElement.style.fontSize).toBe("19px");
    expect(localStorage.getItem("clgo-font-size")).toBe("19");
  });

  it.each([
    ["+", { code: "Minus", key: "+", metaKey: true }, 19],
    ["+ via =", { code: "Minus", key: "=", metaKey: true }, 19],
    ["-", { code: "Equal", key: "-", metaKey: true }, 17],
  ])("suit le caractère %s d'un clavier français", (_label, keyboard, expected) => {
    localStorage.setItem("clgo-font-size", "18");
    const settings = renderShortcutsWithSettings();

    fireEvent.keyDown(window, keyboard);

    expect(settings.result.current.fontSize).toBe(expected);
  });

  it("réduit l'interface avec Ctrl - sans descendre sous la limite", () => {
    localStorage.setItem("clgo-font-size", "11");
    const settings = renderShortcutsWithSettings();

    fireEvent.keyDown(window, { code: "Minus", key: "-", ctrlKey: true });
    fireEvent.keyDown(window, { code: "Minus", key: "-", ctrlKey: true });

    expect(settings.result.current.fontSize).toBe(10);
    expect(document.documentElement.style.fontSize).toBe("10px");
  });

  it("rétablit la taille par défaut avec Cmd 0", () => {
    localStorage.setItem("clgo-font-size", "23");
    const settings = renderShortcutsWithSettings();

    fireEvent.keyDown(window, { code: "Digit0", key: "0", metaKey: true });

    expect(settings.result.current.fontSize).toBe(18);
    expect(document.documentElement.style.fontSize).toBe("18px");
  });

  it("laisse les raccourcis avec Alt disponibles pour les fonctions de l'app", () => {
    localStorage.setItem("clgo-font-size", "18");
    const settings = renderShortcutsWithSettings();

    fireEvent.keyDown(window, {
      altKey: true,
      code: "Equal",
      ctrlKey: true,
      key: "+",
    });

    expect(settings.result.current.fontSize).toBe(18);
  });

  it("ignore les touches de taille sans Ctrl ni Cmd", () => {
    localStorage.setItem("clgo-font-size", "18");
    const settings = renderShortcutsWithSettings();

    fireEvent.keyDown(window, { code: "Equal", key: "+" });

    expect(settings.result.current.fontSize).toBe(18);
  });
});
