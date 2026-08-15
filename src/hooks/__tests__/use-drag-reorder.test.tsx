import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useRef } from "react";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { useDragReorder } from "../use-drag-reorder";

/* Trois cases de 40 pixels espacées de 4 : les milieux tombent sur 20, 64 et
   108. jsdom ne calcule aucune mise en page, on lui dicte donc la sienne. */
const LAYOUT: Record<string, { top: number; height: number }> = {
  a: { top: 0, height: 40 },
  b: { top: 44, height: 40 },
  c: { top: 88, height: 40 },
};

function Reorderable({ ids, onReorder }: { ids: string[]; onReorder: (ids: string[]) => void }) {
  const ref = useRef<HTMLDivElement>(null);
  const drag = useDragReorder({ ids, axis: "y", containerRef: ref, onReorder });
  return (
    <div ref={ref} data-testid="list">
      {drag.order.map((id) => (
        <div key={id} data-testid={`item-${id}`} {...drag.itemProps(id)} {...drag.handleProps(id)}>
          {drag.didDrag() ? "glissé" : "posé"}
        </div>
      ))}
    </div>
  );
}

beforeEach(() => {
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
    this: HTMLElement,
  ) {
    const box = LAYOUT[this.getAttribute("data-drag-id") ?? ""] ?? { top: 0, height: 0 };
    return { top: box.top, left: 0, height: box.height, width: 100 } as DOMRect;
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  cleanup();
});

function grab(item: HTMLElement, y: number) {
  fireEvent.pointerDown(item, { button: 0, clientX: 0, clientY: y });
}

describe("useDragReorder", () => {
  it("n'enregistre rien quand on relâche sans avoir bougé", () => {
    const onReorder = vi.fn();
    const { getByTestId } = render(<Reorderable ids={["a", "b", "c"]} onReorder={onReorder} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerUp(window);

    expect(onReorder).not.toHaveBeenCalled();
  });

  it("ne signale pas de glissement après un simple clic", () => {
    const { getByTestId } = render(<Reorderable ids={["a", "b", "c"]} onReorder={vi.fn()} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerUp(window);

    expect(getByTestId("item-a").textContent).toBe("posé");
  });

  it("n'enregistre rien tant que le milieu du voisin n'est pas franchi", () => {
    const onReorder = vi.fn();
    const { getByTestId } = render(<Reorderable ids={["a", "b", "c"]} onReorder={onReorder} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 40 });
    fireEvent.pointerUp(window);

    expect(onReorder).not.toHaveBeenCalled();
  });

  it("enregistre le nouvel ordre une fois le milieu du voisin franchi", () => {
    const onReorder = vi.fn();
    const { getByTestId } = render(<Reorderable ids={["a", "b", "c"]} onReorder={onReorder} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 56 });
    fireEvent.pointerUp(window);

    expect(onReorder).toHaveBeenCalledWith(["b", "a", "c"], 0, 1);
  });

  it("décale la case tenue et ses voisins pendant le geste", () => {
    const { getByTestId } = render(<Reorderable ids={["a", "b", "c"]} onReorder={vi.fn()} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 56 });

    expect(getByTestId("item-a").style.transform).toBe("translateY(46px)");
    expect(getByTestId("item-b").style.transform).toBe("translateY(-44px)");
    expect(getByTestId("item-c").style.transform).toBe("");
  });

  it("garde le nouvel ordre à l'écran tant que la source n'a pas suivi", () => {
    const { getByTestId, getAllByTestId } = render(
      <Reorderable ids={["a", "b", "c"]} onReorder={vi.fn()} />,
    );

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 56 });
    fireEvent.pointerUp(window);

    const shown = getAllByTestId(/^item-/).map((el) => el.getAttribute("data-drag-id"));
    expect(shown).toEqual(["b", "a", "c"]);
  });

  it("abandonne le geste sans rien enregistrer quand on l'annule", () => {
    const onReorder = vi.fn();
    function Cancellable() {
      const ref = useRef<HTMLDivElement>(null);
      const drag = useDragReorder({ ids: ["a", "b", "c"], axis: "y", containerRef: ref, onReorder });
      return (
        <div ref={ref}>
          <button onClick={drag.cancel}>annuler</button>
          {drag.order.map((id) => (
            <div key={id} data-testid={`item-${id}`} {...drag.itemProps(id)} {...drag.handleProps(id)} />
          ))}
        </div>
      );
    }
    const { getByTestId, getByText } = render(<Cancellable />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 56 });
    fireEvent.click(getByText("annuler"));
    fireEvent.pointerUp(window);

    expect(onReorder).not.toHaveBeenCalled();
    expect(getByTestId("item-a").style.transform).toBe("");
  });
});
