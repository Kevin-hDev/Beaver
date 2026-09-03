/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { PanelSlot, PanelSlotProvider, PanelSlotTarget } from "../panel-slots";

function SlotHarness({ active }: { active: "agent" | "settings" }) {
  return (
    <PanelSlotProvider>
      <section data-nav-zone="list">
        <PanelSlotTarget name="list" />
      </section>
      <section data-nav-zone="detail">
        <PanelSlotTarget name="detail" />
      </section>
      {active === "agent" ? (
        <>
          <PanelSlot name="list">agent list</PanelSlot>
          <PanelSlot name="detail">agent detail</PanelSlot>
        </>
      ) : (
        <>
          <PanelSlot name="list">settings list</PanelSlot>
          <PanelSlot name="detail">settings detail</PanelSlot>
        </>
      )}
    </PanelSlotProvider>
  );
}

describe("Panel slots", () => {
  afterEach(() => cleanup());

  it("rend les contenus dans les panneaux list et detail", () => {
    render(<SlotHarness active="agent" />);

    expect(screen.getByText("agent list").closest("[data-nav-zone]")?.getAttribute("data-nav-zone"))
      .toBe("list");
    expect(screen.getByText("agent detail").closest("[data-nav-zone]")?.getAttribute("data-nav-zone"))
      .toBe("detail");
  });

  it("nettoie l'ancien contenu quand l'onglet change", () => {
    const view = render(<SlotHarness active="agent" />);

    view.rerender(<SlotHarness active="settings" />);

    expect(screen.queryByText("agent list")).toBeNull();
    expect(screen.queryByText("agent detail")).toBeNull();
    expect(screen.getByText("settings list")).toBeTruthy();
    expect(screen.getByText("settings detail")).toBeTruthy();
  });
});
