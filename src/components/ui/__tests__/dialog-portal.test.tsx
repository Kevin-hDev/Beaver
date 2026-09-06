/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useEffect, useState } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { DialogPortal } from "../dialog-portal";

afterEach(cleanup);

function DialogChild({ onMount }: { onMount: () => void }) {
  const [value, setValue] = useState("initial");
  useEffect(() => {
    onMount();
  }, [onMount]);
  return (
    <>
      <output data-testid="dialog-value">{value}</output>
      <button type="button" onClick={() => setValue("dialog conservé")}>update</button>
      <input data-testid="dialog-dom-value" defaultValue="DOM initial" aria-label="dialog DOM" />
    </>
  );
}

describe("portail de dialogue", () => {
  it("reste actif sans provider, avec le contrat du portail de surface", () => {
    let mounts = 0;
    render(
      <DialogPortal>
        <DialogChild onMount={() => { mounts += 1; }} />
      </DialogPortal>,
    );

    const boundary = document.querySelector<HTMLElement>(".app-surface-portal-boundary");
    expect(boundary).not.toHaveAttribute("hidden");
    expect(screen.getByRole("button", { name: "update" })).toBeInTheDocument();
    expect(mounts).toBe(1);
  });

  it("préserve l'état quand le provider rend le dialogue inactif", () => {
    let mounts = 0;
    const onMount = () => { mounts += 1; };
    const view = render(
      <AppSurfaceActivityProvider active>
        <DialogPortal>
          <DialogChild onMount={onMount} />
        </DialogPortal>
      </AppSurfaceActivityProvider>,
    );
    const domInput = document.querySelector<HTMLInputElement>("[data-testid='dialog-dom-value']");

    fireEvent.click(screen.getByRole("button", { name: "update" }));
    fireEvent.change(domInput!, { target: { value: "DOM dialog conservé" } });
    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <DialogPortal>
          <DialogChild onMount={onMount} />
        </DialogPortal>
      </AppSurfaceActivityProvider>,
    );

    expect(screen.queryByRole("button", { name: "update" })).toBeNull();
    expect(mounts).toBe(1);

    view.rerender(
      <AppSurfaceActivityProvider active>
        <DialogPortal>
          <DialogChild onMount={onMount} />
        </DialogPortal>
      </AppSurfaceActivityProvider>,
    );

    expect(screen.getByTestId("dialog-value")).toHaveTextContent("dialog conservé");
    expect(screen.getByTestId("dialog-dom-value")).toHaveValue("DOM dialog conservé");
    expect(mounts).toBe(1);
  });
});
