import { act, cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Tooltip } from "../tooltip";

afterEach(() => {
  cleanup();
  restoreMeasurements();
});

/* jsdom ne fait pas de mise en page : sans ces mesures, tout élément est un
   point en (0,0) et le placement ne peut pas être vérifié. */
const BUBBLE_WIDTH = 120;
const BUBBLE_HEIGHT = 24;
const restorers: Array<() => void> = [];

function restoreMeasurements() {
  while (restorers.length) restorers.pop()?.();
}

function measureBubble() {
  for (const [property, value] of [["offsetWidth", BUBBLE_WIDTH], ["offsetHeight", BUBBLE_HEIGHT]] as const) {
    const original = Object.getOwnPropertyDescriptor(HTMLElement.prototype, property);
    Object.defineProperty(HTMLElement.prototype, property, { configurable: true, value });
    restorers.push(() => {
      if (original) Object.defineProperty(HTMLElement.prototype, property, original);
    });
  }
}

/* Posé sur l'élément lui-même : la bulle n'existe qu'à l'ouverture, on ne peut
   pas la préparer d'avance. */
function anchorAt(container: HTMLElement, top: number, left: number, width = 40, height = 20) {
  const rect = {
    top, left, width, height,
    bottom: top + height,
    right: left + width,
    x: left, y: top,
    toJSON: () => ({}),
  } as DOMRect;
  Object.defineProperty(container.querySelector(".tooltip-wrapper")!, "getBoundingClientRect", {
    configurable: true,
    value: () => rect,
  });
}

function openTooltip(container: HTMLElement) {
  act(() => {
    fireEvent.mouseEnter(container.querySelector(".tooltip-wrapper")!);
  });
  act(() => {
    vi.advanceTimersByTime(300);
  });
}

describe("Tooltip", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    window.innerHeight = 800;
    window.innerWidth = 1200;
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

    openTooltip(container);

    expect(getByText("Mon aide")).toBeTruthy();
  });

  it("cache la bulle au départ de la souris", () => {
    const { container, queryByText } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );

    openTooltip(container);
    act(() => {
      fireEvent.mouseLeave(container.querySelector(".tooltip-wrapper")!);
    });

    expect(queryByText("Mon aide")).toBeNull();
  });

  /* Un panneau qui rogne son débordement coupe toute bulle rendue dans son
     flux : la bulle doit sortir de son parent pour se poser sur le document. */
  it("pose la bulle sur le document et non dans son parent", () => {
    const { container } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );

    openTooltip(container);
    const bubble = document.querySelector(".tooltip-bubble");

    expect(bubble?.textContent).toBe("Mon aide");
    expect(container.contains(bubble)).toBe(false);
    const boundary = bubble?.parentElement;
    expect(boundary).toHaveClass("app-surface-portal-boundary");
    expect(boundary?.parentElement).toBe(document.body);
  });

  it("ouvre la bulle en dessous quand la fenêtre laisse la place", () => {
    const { container } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );
    measureBubble();
    anchorAt(container, 100, 300);

    openTooltip(container);
    const bubble = document.querySelector<HTMLElement>(".tooltip-bubble")!;

    expect(bubble.className).toContain("tooltip-below");
    expect(bubble.style.top).toBe("126px");
    expect(bubble.style.bottom).toBe("");
  });

  /* Le cas des boutons de la zone de saisie : au ras du bas de la fenêtre, une
     bulle ouverte vers le bas est coupée par le bord de la vue. */
  it("bascule au-dessus quand le bas de la fenêtre ne laisse pas la place", () => {
    const { container } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );
    measureBubble();
    anchorAt(container, 770, 300);

    openTooltip(container);
    const bubble = document.querySelector<HTMLElement>(".tooltip-bubble")!;

    expect(bubble.className).toContain("tooltip-above");
    expect(bubble.style.bottom).toBe("36px");
    expect(bubble.style.top).toBe("");
  });

  it("aligne la bulle sur le bord droit de l'élément en alignement right", () => {
    const { container } = render(
      <Tooltip label="Mon aide" align="right">
        <button>action</button>
      </Tooltip>,
    );
    measureBubble();
    anchorAt(container, 100, 300);

    openTooltip(container);
    const bubble = document.querySelector<HTMLElement>(".tooltip-bubble")!;

    expect(bubble.style.left).toBe("220px");
  });

  /* Une bulle plus large que son élément sort de l'écran en bord de fenêtre :
     le bornage passe avant l'alignement demandé. */
  it("garde la bulle dans la fenêtre quand l'élément touche le bord droit", () => {
    const { container } = render(
      <Tooltip label="Mon aide">
        <button>action</button>
      </Tooltip>,
    );
    measureBubble();
    anchorAt(container, 100, 1180, 20);

    openTooltip(container);
    const bubble = document.querySelector<HTMLElement>(".tooltip-bubble")!;

    expect(bubble.style.left).toBe("1072px");
  });
});
