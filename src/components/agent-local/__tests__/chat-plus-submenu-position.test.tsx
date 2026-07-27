import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { useChatPlusSubmenuPosition } from "../use-chat-plus-submenu-position";

const initialInnerWidth = window.innerWidth;

function element(width: number, left: number) {
  const node = document.createElement("div");
  Object.defineProperty(node, "offsetWidth", { value: width });
  node.getBoundingClientRect = () => ({
    width,
    left,
    right: left + width,
    top: 0,
    bottom: 0,
    height: 0,
    x: left,
    y: 0,
    toJSON: () => ({}),
  });
  return node;
}

describe("useChatPlusSubmenuPosition", () => {
  afterEach(() => {
    document.documentElement.style.removeProperty("--space-xs");
    Object.defineProperty(window, "innerWidth", {
      value: initialInnerWidth,
      configurable: true,
    });
  });

  it("ouvre à droite quand le sous-menu tient dans la fenêtre", async () => {
    document.documentElement.style.setProperty("--space-xs", "4px");
    const wrapper = { current: element(28, 20) };
    const dropdown = { current: element(240, 20) };
    const submenu = { current: element(220, 0) };

    const view = renderHook(() =>
      useChatPlusSubmenuPosition(true, "plugins", wrapper, dropdown, submenu));

    await waitFor(() => expect(view.result.current).toBe(244));
  });

  it("bascule à gauche avant de dépasser le viewport", async () => {
    document.documentElement.style.setProperty("--space-xs", "4px");
    Object.defineProperty(window, "innerWidth", { value: 900, configurable: true });
    const wrapper = { current: element(28, 600) };
    const dropdown = { current: element(240, 600) };
    const submenu = { current: element(220, 0) };

    const view = renderHook(() =>
      useChatPlusSubmenuPosition(true, "plugins", wrapper, dropdown, submenu));

    await waitFor(() => expect(view.result.current).toBe(-224));
  });
});
