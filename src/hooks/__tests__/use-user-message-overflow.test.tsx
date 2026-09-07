/* @vitest-environment jsdom */
import { act, render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { useUserMessageOverflow } from "../use-user-message-overflow";

function Wrapper({ active, children }: { active: boolean; children: React.ReactNode }) {
  return <AppSurfaceActivityProvider active={active}>{children}</AppSurfaceActivityProvider>;
}

function Probe() {
  const { contentRef, maxHeight } = useUserMessageOverflow("message long", false);
  return <div ref={contentRef} data-max-height={maxHeight ?? "none"} />;
}

describe("useUserMessageOverflow et l'activité", () => {
  it("ignore une mesure nulle inactive et remesure au retour", () => {
    let active = true;
    const view = render(<Wrapper active={active}><Probe /></Wrapper>);
    const element = view.container.firstElementChild as HTMLDivElement;
    Object.defineProperty(element, "scrollHeight", { configurable: true, value: 120 });
    void act(() => window.dispatchEvent(new Event("resize")));
    const valid = element.dataset.maxHeight;

    active = false;
    view.rerender(<Wrapper active={active}><Probe /></Wrapper>);
    Object.defineProperty(element, "scrollHeight", { configurable: true, value: 0 });
    void act(() => window.dispatchEvent(new Event("resize")));
    expect(element.dataset.maxHeight).toBe(valid);

    active = true;
    view.rerender(<Wrapper active={active}><Probe /></Wrapper>);
    expect(element.dataset.maxHeight).toBe(valid);
  });
});
