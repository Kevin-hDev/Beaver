/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useEffect, useState } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { AppSurfacePortal } from "../app-surface-portal";

afterEach(cleanup);

function StatefulPortalChild({ onMount }: { onMount: () => void }) {
  const [value, setValue] = useState("initial");
  useEffect(() => {
    onMount();
  }, [onMount]);

  return (
    <>
      <output data-testid="react-value">{value}</output>
      <button type="button" onClick={() => setValue("React conservé")}>update</button>
      <input data-testid="dom-value" defaultValue="DOM initial" aria-label="valeur DOM" />
    </>
  );
}

describe("portail d'une surface applicative", () => {
  it("conserve le portail monté et son état pendant l'inactivité", () => {
    let mounts = 0;
    const onMount = () => { mounts += 1; };
    const view = render(
      <AppSurfaceActivityProvider active>
        <AppSurfacePortal>
          <StatefulPortalChild onMount={onMount} />
        </AppSurfacePortal>
      </AppSurfaceActivityProvider>,
    );
    const boundary = document.querySelector<HTMLElement>(".app-surface-portal-boundary");
    const domInput = document.querySelector<HTMLInputElement>("[data-testid='dom-value']");

    expect(boundary).toBeTruthy();
    expect(boundary).not.toHaveAttribute("hidden");
    fireEvent.click(screen.getByRole("button", { name: "update" }));
    fireEvent.change(domInput!, { target: { value: "DOM conservé" } });

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <AppSurfacePortal>
          <StatefulPortalChild onMount={onMount} />
        </AppSurfacePortal>
      </AppSurfaceActivityProvider>,
    );

    expect(boundary).toHaveAttribute("hidden");
    expect(boundary).toHaveAttribute("aria-hidden", "true");
    expect(boundary).toHaveAttribute("inert");
    expect(screen.queryByRole("button", { name: "update" })).toBeNull();
    expect(mounts).toBe(1);

    view.rerender(
      <AppSurfaceActivityProvider active>
        <AppSurfacePortal>
          <StatefulPortalChild onMount={onMount} />
        </AppSurfacePortal>
      </AppSurfaceActivityProvider>,
    );

    expect(screen.getByTestId("react-value")).toHaveTextContent("React conservé");
    expect(screen.getByTestId("dom-value")).toHaveValue("DOM conservé");
    expect(mounts).toBe(1);
  });
});
