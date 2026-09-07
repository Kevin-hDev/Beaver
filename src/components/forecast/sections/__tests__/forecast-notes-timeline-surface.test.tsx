/* @vitest-environment jsdom */
import { render } from "@testing-library/react";
import type { TFunction } from "i18next";
import { describe, expect, it } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { ForecastNotesTimeline } from "../forecast-notes-timeline";

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
});
