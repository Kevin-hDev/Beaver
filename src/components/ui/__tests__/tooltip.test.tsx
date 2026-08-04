import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Tooltip } from "../tooltip";

afterEach(() => cleanup());

describe("Tooltip", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("rend le children sans afficher la bulle au repos", () => {
    const { container, queryByText } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );

    expect(container.querySelector("button")?.textContent).toBe("action");
    expect(queryByText("Mon aide")).toBeNull();
  });

  it("affiche le label après le délai au survol", () => {
    const { container, getByText } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );

    act(() => {
      fireEvent.mouseEnter(container.querySelector(".tooltip-wrapper")!);
    });
    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(getByText("Mon aide")).toBeTruthy();
  });

  it("cache la bulle au départ de la souris", () => {
    const { container, queryByText } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );

    const wrapper = container.querySelector(".tooltip-wrapper")!;
    act(() => {
      fireEvent.mouseEnter(wrapper);
    });
    act(() => {
      vi.advanceTimersByTime(300);
    });
    act(() => {
      fireEvent.mouseLeave(wrapper);
    });

    expect(queryByText("Mon aide")).toBeNull();
  });

  it("applique l'alignement right via la classe tooltip-right", () => {
    const { container } = render(
      <Tooltip label="Mon aide" align="right">
        <button>action</button>
      </Tooltip>,
    );

    act(() => {
      fireEvent.mouseEnter(container.querySelector(".tooltip-wrapper")!);
    });
    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(container.querySelector(".tooltip-right")).toBeTruthy();
  });

  /* Une bulle placée au-dessus sert un élément posé au bas d'un panneau, et ce
     panneau rogne son débordement : rendue dans le flux, elle serait coupée.
     Elle doit donc sortir de son parent pour se poser sur le document. */
  it("pose la bulle du dessus sur le document et non dans son parent", () => {
    const { container } = render(
      <Tooltip label="Mon aide" placement="top">
        <button>action</button>
      </Tooltip>,
    );

    act(() => {
      fireEvent.mouseEnter(container.querySelector(".tooltip-wrapper")!);
    });
    act(() => {
      vi.advanceTimersByTime(300);
    });

    const bubble = document.querySelector(".tooltip-above");

    expect(bubble).toBeTruthy();
    expect(bubble?.textContent).toBe("Mon aide");
    expect(bubble?.parentElement).toBe(document.body);
    expect(container.querySelector(".tooltip-above")).toBeNull();
  });

  it("garde la bulle dans le flux quand elle s'ouvre vers le bas", () => {
    const { container } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );

    act(() => {
      fireEvent.mouseEnter(container.querySelector(".tooltip-wrapper")!);
    });
    act(() => {
      vi.advanceTimersByTime(300);
    });

    expect(container.querySelector(".tooltip-bubble")).toBeTruthy();
    expect(document.querySelector(".tooltip-above")).toBeNull();
  });
});
