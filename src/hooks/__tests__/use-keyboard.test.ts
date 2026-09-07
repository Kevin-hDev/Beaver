/* @vitest-environment jsdom */
import { cleanup, fireEvent, render } from "@testing-library/react";
import { createElement } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { useKeyboard } from "../use-keyboard";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function KeyboardProbe({ onEscape }: { onEscape: () => void }) {
  useKeyboard({ onEscape });
  return createElement("div", null, "keyboard");
}

function Harness({ active, onEscape }: { active: boolean; onEscape: () => void }) {
  return (
    createElement(
      AppSurfaceActivityProvider,
      { active } as never,
      createElement(KeyboardProbe, { onEscape }),
    )
  );
}

describe("useKeyboard et l'activité de surface", () => {
  it("retire le listener inactif et le réinstalle une seule fois au retour", () => {
    const onEscape = vi.fn();
    const add = vi.spyOn(document, "addEventListener");
    const remove = vi.spyOn(document, "removeEventListener");
    const view = render(createElement(Harness, { active: true, onEscape }));

    expect(add.mock.calls.filter(([type]) => type === "keydown")).toHaveLength(1);
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onEscape).toHaveBeenCalledTimes(1);

    view.rerender(createElement(Harness, { active: false, onEscape }));
    expect(remove.mock.calls.filter(([type]) => type === "keydown")).toHaveLength(1);
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onEscape).toHaveBeenCalledTimes(1);

    view.rerender(createElement(Harness, { active: true, onEscape }));
    expect(add.mock.calls.filter(([type]) => type === "keydown")).toHaveLength(2);
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onEscape).toHaveBeenCalledTimes(2);
  });
});
