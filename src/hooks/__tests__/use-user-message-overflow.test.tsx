/* @vitest-environment jsdom */
import { act, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { useUserMessageOverflow } from "../use-user-message-overflow";

function Wrapper({ active, children }: { active: boolean; children: React.ReactNode }) {
  return <AppSurfaceActivityProvider active={active}>{children}</AppSurfaceActivityProvider>;
}

function Probe({ expanded = false }: { expanded?: boolean }) {
  const { contentRef, maxHeight } = useUserMessageOverflow("message long", expanded);
  return <div ref={contentRef} data-max-height={maxHeight ?? "none"} />;
}

beforeEach(() => {
  document.documentElement.style.fontSize = "14px";
});

afterEach(() => {
  document.documentElement.style.removeProperty("font-size");
});

describe("useUserMessageOverflow et l'activité", () => {
  it("régression: conserve une mesure réelle inactive et la remesure au retour", async () => {
    let active = true;
    const view = render(<Wrapper active={active}><Probe /></Wrapper>);
    const element = view.container.firstElementChild as HTMLDivElement;
    Object.defineProperty(element, "scrollHeight", { configurable: true, value: 600 });
    void act(() => window.dispatchEvent(new Event("resize")));
    await waitFor(() => expect(element.dataset.maxHeight).toBe("434px"));
    const valid = element.dataset.maxHeight;
    expect(valid).toMatch(/^\d+px$/);

    active = false;
    view.rerender(<Wrapper active={active}><Probe /></Wrapper>);
    Object.defineProperty(element, "scrollHeight", { configurable: true, value: 0 });
    void act(() => window.dispatchEvent(new Event("resize")));
    expect(element.dataset.maxHeight).toBe(valid);

    active = true;
    Object.defineProperty(element, "scrollHeight", { configurable: true, value: 800 });
    view.rerender(<Wrapper active={active}><Probe /></Wrapper>);
    await waitFor(() => expect(element.dataset.maxHeight).toBe("434px"));
    view.rerender(<Wrapper active={active}><Probe expanded /></Wrapper>);
    await waitFor(() => expect(element.dataset.maxHeight).toBe("800px"));
  });
});
