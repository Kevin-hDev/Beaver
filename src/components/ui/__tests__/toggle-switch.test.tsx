import { readFileSync } from "node:fs";
import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ToggleSwitch } from "../toggle-switch";

const switchCss = readFileSync("src/components/ui/toggle-switch.css", "utf8");

describe("ToggleSwitch", () => {
  it("expose un switch accessible avec son état", () => {
    const view = render(
      <ToggleSwitch
        checked
        ariaLabel="Activer la fonctionnalité"
        onCheckedChange={() => undefined}
      />,
    );

    const control = view.getByRole("switch", { name: "Activer la fonctionnalité" });
    expect(control).toBeChecked();
  });

  it("transmet le nouvel état", () => {
    const onCheckedChange = vi.fn();
    const view = render(
      <ToggleSwitch
        checked={false}
        ariaLabel="Activer la fonctionnalité"
        onCheckedChange={onCheckedChange}
      />,
    );

    fireEvent.click(view.getByRole("switch"));

    expect(onCheckedChange).toHaveBeenCalledWith(true);
  });

  it("gère l'état désactivé", () => {
    const view = render(
      <ToggleSwitch
        checked={false}
        ariaLabel="Activer la fonctionnalité"
        disabled
        onCheckedChange={() => undefined}
      />,
    );

    expect(view.getByRole("switch")).toBeDisabled();
    expect(view.container.querySelector(".uis-switch-disabled")).toBeTruthy();
  });

  it("conserve la forme rectangulaire arrondie fournie", () => {
    expect(switchCss).toMatch(/\.uis-slider\s*\{[^}]*border-radius:\s*0\.5em;/s);
    expect(switchCss).toMatch(/\.uis-slider::before\s*\{[^}]*border-radius:\s*0\.25em;/s);
  });

  it("centre verticalement le curseur dans les deux états", () => {
    expect(switchCss).toMatch(/\.uis-slider::before\s*\{[^}]*top:\s*50%;/s);
    expect(switchCss).toMatch(/\.uis-slider::before\s*\{[^}]*transform:\s*translateY\(-50%\);/s);
    expect(switchCss).toContain("transform: translate(1.5em, -50%);");
  });
});
