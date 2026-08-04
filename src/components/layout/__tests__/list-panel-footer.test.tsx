/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ListPanelFooter } from "../list-panel-footer";
import { NAV_ITEMS } from "../nav-items";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

/* Le badge lit l'état GPU par le pont Tauri, absent des tests. La rangée le
   place, elle ne le décrit pas : on le neutralise pour ne tester qu'elle. */
vi.mock("@/components/agent-local/gpu-status-badge", () => ({
  GpuStatusBadge: () => <span data-testid="gpu-badge" />,
}));

afterEach(() => {
  cleanup();
});

describe("rangée de navigation du panneau liste", () => {
  it("expose une entrée par section", () => {
    render(<ListPanelFooter activeTab="agent-local" onTabChange={() => {}} />);

    expect(screen.getAllByRole("button")).toHaveLength(NAV_ITEMS.length);
    for (const item of NAV_ITEMS) {
      expect(screen.getByLabelText(item.i18nKey)).toBeTruthy();
    }
  });

  it("inclut les Réglages parmi les sections", () => {
    /* Ils vivaient à part dans le rail, détachés en bas d'une colonne. Rien ne
       doit les en distinguer maintenant qu'ils ferment la rangée. */
    expect(NAV_ITEMS.map((item) => item.id)).toContain("settings");
  });

  it("marque la section active pour la mise au point clavier", () => {
    /* App.tsx vise [data-nav-zone="sidebar"] [data-nav-active="true"] : les deux
       attributs doivent survivre au passage du rail à la rangée. */
    render(<ListPanelFooter activeTab="heartbeat" onTabChange={() => {}} />);

    const zone = document.querySelector('[data-nav-zone="sidebar"]');
    const active = zone?.querySelectorAll('[data-nav-active="true"]');

    expect(active).toHaveLength(1);
    expect(active?.[0].getAttribute("aria-label")).toBe("nav.heartbeat");
  });

  it("signale la section demandée au clic", () => {
    const onTabChange = vi.fn();
    render(<ListPanelFooter activeTab="agent-local" onTabChange={onTabChange} />);

    fireEvent.click(screen.getByLabelText("nav.personality"));

    expect(onTabChange).toHaveBeenCalledWith("personality");
  });

  it("garde le badge GPU sur la rangée", () => {
    render(<ListPanelFooter activeTab="agent-local" onTabChange={() => {}} />);

    expect(screen.getByTestId("gpu-badge")).toBeTruthy();
  });
});
