/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useEffect, useState } from "react";
import { afterEach, describe, expect, it } from "vitest";
import {
  AppSurfaceActivityProvider,
} from "../app-surface-activity";
import { AppSurfaceBoundary } from "../app-surface-boundary";

afterEach(cleanup);

function StatefulChild({ onMount }: { onMount: () => void }) {
  const [value, setValue] = useState("initial");
  useEffect(() => onMount(), [onMount]);

  return (
    <>
      <input
        aria-label="valeur"
        value={value}
        onChange={(event) => setValue(event.target.value)}
      />
      <button type="button">focus</button>
    </>
  );
}

describe("frontière d'une surface applicative", () => {
  it("masque une surface inactive sans démonter ni perdre son état", () => {
    let mountCount = 0;
    const onMount = () => {
      mountCount += 1;
    };
    const view = render(
      <AppSurfaceActivityProvider active>
        <AppSurfaceBoundary>
          <StatefulChild onMount={onMount} />
        </AppSurfaceBoundary>
      </AppSurfaceActivityProvider>,
    );
    const boundary = screen.getByRole("button", { name: "focus" }).parentElement;
    const input = screen.getByRole("textbox", { name: "valeur" });

    expect(boundary).not.toHaveAttribute("hidden");
    expect(boundary).not.toHaveAttribute("aria-hidden");
    expect(boundary).not.toHaveAttribute("inert");

    fireEvent.change(input, { target: { value: "conservée" } });
    expect(mountCount).toBe(1);

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <AppSurfaceBoundary>
          <StatefulChild onMount={onMount} />
        </AppSurfaceBoundary>
      </AppSurfaceActivityProvider>,
    );

    expect(boundary).toHaveAttribute("hidden");
    expect(boundary).toHaveAttribute("aria-hidden", "true");
    expect(boundary).toHaveAttribute("inert");
    expect(mountCount).toBe(1);

    view.rerender(
      <AppSurfaceActivityProvider active>
        <AppSurfaceBoundary>
          <StatefulChild onMount={onMount} />
        </AppSurfaceBoundary>
      </AppSurfaceActivityProvider>,
    );

    expect(screen.getByRole("textbox", { name: "valeur" })).toHaveValue("conservée");
    expect(mountCount).toBe(1);
  });

  it("retire le focus d'un descendant au moment de la désactivation", () => {
    const view = render(
      <AppSurfaceActivityProvider active>
        <AppSurfaceBoundary>
          <button type="button">focus</button>
        </AppSurfaceBoundary>
      </AppSurfaceActivityProvider>,
    );
    const button = screen.getByRole("button", { name: "focus" });
    button.focus();
    expect(button).toHaveFocus();

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <AppSurfaceBoundary>
          <button type="button">focus</button>
        </AppSurfaceBoundary>
      </AppSurfaceActivityProvider>,
    );

    const boundary = document.querySelector<HTMLElement>("[hidden]");
    const activeElement = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    expect(boundary).toBeTruthy();
    expect(boundary).not.toContainElement(activeElement);
  });
});
