import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { BROWSER_NATIVE_OCCLUSION_EVENT } from "@/components/internal-browser/browser-native-occlusion";
import { ModeSelector } from "../mode-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      "forecast.panelMode.title": "Changer de mode",
      "forecast.panelMode.preview": "Aperçu",
      "forecast.panelMode.forecast": "Forecast",
      "browser.title": "Navigateur",
      "browser.unavailable": "Navigateur sécurisé indisponible",
    }[key] ?? key),
  }),
}));

describe("ModeSelector browser mode", () => {
  afterEach(() => vi.restoreAllMocks());

  it("affiche le navigateur prêt avec un préfixe CSS réservé", () => {
    render(
      <ModeSelector
        mode="preview"
        browserStatus="ready"
        onChange={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button"));

    expect(screen.getByText("Navigateur")).toBeTruthy();
    expect(document.body.querySelector(".asp-mode-menu")).toBeTruthy();
    expect(document.body.querySelector(".ms-menu")).toBeNull();
  });

  it("montre le navigateur indisponible au lieu de le masquer", () => {
    const onChange = vi.fn();
    render(
      <ModeSelector
        mode="preview"
        browserStatus="unavailable"
        onChange={onChange}
      />,
    );

    fireEvent.click(screen.getByRole("button"));

    const browser = screen.getByRole("button", { name: "Navigateur" });
    expect(browser).toBeDisabled();
    expect(browser).toHaveAttribute("title", "Navigateur sécurisé indisponible");
    expect(onChange).not.toHaveBeenCalled();
  });

  it("n'occulte pas la surface CEF quand le menu s'ouvre", () => {
    const onOcclusion = vi.fn();
    window.addEventListener(BROWSER_NATIVE_OCCLUSION_EVENT, onOcclusion);

    render(
      <ModeSelector
        mode="browser"
        browserStatus="ready"
        onChange={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByRole("button"));

    expect(onOcclusion).not.toHaveBeenCalled();
    window.removeEventListener(BROWSER_NATIVE_OCCLUSION_EVENT, onOcclusion);
  });

  it("place le menu au-dessus de la surface native sans la recouvrir", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (this: HTMLElement) {
      if (this.classList.contains("ib-surface")) {
        return rect({ left: 500, top: 120, width: 500, height: 580 });
      }
      if (this.classList.contains("asp-mode-menu")) {
        return rect({ left: 840, top: 46, width: 160, height: 100 });
      }
      if (this instanceof HTMLButtonElement) {
        return rect({ left: 1_000, top: 10, width: 40, height: 30 });
      }
      return rect({ left: 0, top: 0, width: 0, height: 0 });
    });

    render(
      <>
        <div className="ib-surface" data-native-active="true" />
        <ModeSelector mode="browser" browserStatus="ready" onChange={vi.fn()} />
      </>,
    );
    fireEvent.click(screen.getByRole("button"));

    const menu = document.body.querySelector<HTMLElement>(".asp-mode-menu");
    await waitFor(() => expect(Number.parseFloat(menu?.style.top ?? "NaN")).toBeLessThanOrEqual(16));
  });

  it("reste aligné au bouton quand l'onglet navigateur est vide", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (this: HTMLElement) {
      if (this.classList.contains("ib-surface")) {
        return rect({ left: 500, top: 120, width: 500, height: 580 });
      }
      if (this.classList.contains("asp-mode-menu")) {
        return rect({ left: 680, top: 44, width: 160, height: 100 });
      }
      if (this instanceof HTMLButtonElement) {
        return rect({ left: 800, top: 10, width: 40, height: 30 });
      }
      return rect({ left: 0, top: 0, width: 0, height: 0 });
    });

    render(
      <>
        <div className="ib-surface" data-native-active="false" />
        <ModeSelector mode="browser" browserStatus="ready" onChange={vi.fn()} />
      </>,
    );
    fireEvent.click(screen.getByRole("button"));

    const menu = document.body.querySelector<HTMLElement>(".asp-mode-menu");
    await waitFor(() => expect(menu?.style.top).toBe("44px"));
    expect(menu?.style.right).toBe("184px");
  });
});

function rect(values: { left: number; top: number; width: number; height: number }): DOMRect {
  const { left, top, width, height } = values;
  return {
    x: left,
    y: top,
    left,
    top,
    width,
    height,
    right: left + width,
    bottom: top + height,
    toJSON: () => ({}),
  };
}
