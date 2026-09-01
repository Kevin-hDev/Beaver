/* @vitest-environment jsdom */
// Finding 7 : replié, le corps de la carte doit sortir de l'ordre de
// tabulation et de l'arbre d'accessibilité tout en restant dans la page pour
// pouvoir s'animer. Le dépliement lui-même est confié à <Collapsible>.
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ForecastChartCard } from "../forecast-chart-card";

vi.mock("react-i18next", async (importOriginal) => {
  const actual = await importOriginal<typeof import("react-i18next")>();
  return {
    ...actual,
    useTranslation: () => ({ t: (key: string) => key }),
  };
});

afterEach(cleanup);

describe("ForecastChartCard collapsed a11y", () => {
  it("rend le corps replie inerte et cache aux technologies d'assistance", () => {
    const { container } = render(
      <ForecastChartCard title="card" defaultOpen={false}>
        <button type="button">focusable</button>
      </ForecastChartCard>,
    );
    const body = container.querySelector(".fcrd-body");
    expect(body).toBeTruthy();
    expect(body!.hasAttribute("inert")).toBe(true);
    expect(body!.getAttribute("aria-hidden")).toBe("true");
  });

  it("retire inert et aria-hidden une fois la carte depliee", () => {
    const { container } = render(
      <ForecastChartCard title="card" defaultOpen={false}>
        <button type="button">focusable</button>
      </ForecastChartCard>,
    );
    fireEvent.click(container.querySelector(".fcrd-toggle")!);
    const body = container.querySelector(".fcrd-body");
    expect(body!.hasAttribute("inert")).toBe(false);
    expect(body!.getAttribute("aria-hidden")).toBe("false");
  });
});
