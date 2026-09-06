/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { useEffect } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  AppSurfaceActivityProvider,
  useAppSurfaceActive,
} from "../app-surface-activity";

afterEach(cleanup);

function ActivityProbe() {
  return <output data-testid="activity">{String(useAppSurfaceActive())}</output>;
}

describe("activité d'une surface applicative", () => {
  it("est active par défaut hors provider", () => {
    render(<ActivityProbe />);

    expect(screen.getByTestId("activity")).toHaveTextContent("true");
  });

  it("publie l'activité du provider sans remonter ses enfants", () => {
    const mounts = vi.fn<() => void>();

    function Child() {
      useEffect(() => {
        mounts();
      }, []);
      return <ActivityProbe />;
    }

    const view = render(
      <AppSurfaceActivityProvider active={false}>
        <Child />
      </AppSurfaceActivityProvider>,
    );

    expect(screen.getByTestId("activity")).toHaveTextContent("false");
    expect(mounts).toHaveBeenCalledTimes(1);

    view.rerender(
      <AppSurfaceActivityProvider active>
        <Child />
      </AppSurfaceActivityProvider>,
    );

    expect(screen.getByTestId("activity")).toHaveTextContent("true");
    expect(mounts).toHaveBeenCalledTimes(1);
  });
});
