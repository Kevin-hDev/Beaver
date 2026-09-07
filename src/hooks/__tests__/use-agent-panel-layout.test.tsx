/* @vitest-environment jsdom */
import { createElement } from "react";
import { act, render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { useAgentPanelLayout } from "../use-agent-panel-layout";

function Wrapper({ active, children }: { active: boolean; children: React.ReactNode }) {
  return createElement(AppSurfaceActivityProvider, { active } as never, children);
}

function Probe() {
  const { containerRef, layout } = useAgentPanelLayout({
    previewOpen: true,
    previewFullscreen: false,
    previewDesiredWidth: 320,
    fileTreeOpen: false,
    fileTreeDesiredWidth: 240,
  });
  return <div ref={containerRef} data-preview-width={layout.previewWidth} />;
}

describe("useAgentPanelLayout et l'activité", () => {
  it("conserve la dernière mesure pendant l'inactivité puis remesure au retour", () => {
    let active = true;
    const view = render(<Wrapper active={active}><Probe /></Wrapper>);
    const container = view.container.firstElementChild as HTMLDivElement;
    container.getBoundingClientRect = () => ({
      width: 1000, height: 0, top: 0, right: 1000, bottom: 0, left: 0, x: 0, y: 0,
      toJSON: () => ({}),
    });
    void act(() => window.dispatchEvent(new Event("resize")));
    const measured = Number(container.dataset.previewWidth);

    active = false;
    view.rerender(<Wrapper active={active}><Probe /></Wrapper>);
    container.getBoundingClientRect = () => ({
      width: 300, height: 0, top: 0, right: 300, bottom: 0, left: 0, x: 0, y: 0,
      toJSON: () => ({}),
    });
    void act(() => window.dispatchEvent(new Event("resize")));
    expect(Number(container.dataset.previewWidth)).toBe(measured);

    active = true;
    view.rerender(<Wrapper active={active}><Probe /></Wrapper>);
    expect(Number(container.dataset.previewWidth)).not.toBe(measured);
  });
});
