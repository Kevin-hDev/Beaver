/* @vitest-environment jsdom */
import { act, render, waitFor } from "@testing-library/react";
import type { TFunction } from "i18next";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { ForecastNotesTimeline } from "../forecast-notes-timeline";
import type { ForecastNote } from "../forecast-notes-types";

function note(id: string, date: string): ForecastNote {
  return {
    id,
    analysis_id: "analysis",
    date,
    title: id,
    note_type: "note",
    source: "user",
    content: "",
    file_path: "",
    created_at: date,
    updated_at: date,
  };
}

const RANGE = { start: Date.parse("2026-01-01"), end: Date.parse("2026-12-31") };
const NOTES = [
  note("march", "2026-03-01"),
  note("april", "2026-04-15"),
  note("june", "2026-06-01"),
  note("july", "2026-07-15"),
  note("september", "2026-09-01"),
];

describe("ForecastNotesTimeline et l'activité", () => {
  it("reste montée quand sa surface devient inactive", () => {
    const view = render(
      <AppSurfaceActivityProvider active={false}>
        <ForecastNotesTimeline
          notes={[]}
          selectedId={null}
          range={{ start: Date.parse("2026-01-01"), end: Date.parse("2026-12-31") }}
          height={120}
          locale="fr-FR"
          t={((key: string) => key) as TFunction}
          onSelect={() => {}}
        />
      </AppSurfaceActivityProvider>,
    );
    expect(view.container.querySelector(".fcn-timeline")).toBeTruthy();
  });

  it("régression: conserve une largeur positive inactive et la remesure au retour", async () => {
    let measuredWidth = 600;
    let notifyResize: (() => void) | null = null;
    class TestResizeObserver {
      private active = true;

      constructor(callback: ResizeObserverCallback) {
        notifyResize = () => {
          if (this.active) callback([], this as unknown as ResizeObserver);
        };
      }
      observe() {}
      disconnect() {
        this.active = false;
      }
    }
    vi.stubGlobal("ResizeObserver", TestResizeObserver);
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(() => ({
      width: measuredWidth,
    } as DOMRect));

    const renderTimeline = (active: boolean) => (
      <AppSurfaceActivityProvider active={active}>
        <ForecastNotesTimeline
          notes={NOTES}
          selectedId={null}
          range={RANGE}
          height={120}
          locale="fr-FR"
          t={((key: string) => key) as TFunction}
          onSelect={() => {}}
        />
      </AppSurfaceActivityProvider>
    );
    const view = render(renderTimeline(true));
    const markerCount = () => view.container.querySelectorAll(".fcn-axis-mark").length;
    const initialCount = markerCount();
    expect(initialCount).toBeGreaterThan(0);

    measuredWidth = 300;
    view.rerender(renderTimeline(false));
    void act(() => notifyResize?.());
    expect(markerCount()).toBe(initialCount);

    measuredWidth = 0;
    view.rerender(renderTimeline(true));
    expect(markerCount()).toBe(initialCount);

    measuredWidth = 300;
    void act(() => notifyResize?.());
    await waitFor(() => expect(markerCount()).toBeLessThan(initialCount));
    expect(markerCount()).toBeGreaterThan(0);
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});
