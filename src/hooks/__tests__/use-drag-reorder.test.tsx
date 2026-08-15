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
  /* Deux listes imbriquées dans un même conteneur : « p » et « q » à l'une,
     « r » posée entre les deux à l'autre. */
  p: { top: 0, height: 40 },
  r: { top: 44, height: 40 },
  q: { top: 88, height: 40 },
};

function Reorderable({ ids, onReorder }: { ids: string[]; onReorder: (ids: string[]) => void }) {
  const ref = useRef<HTMLDivElement>(null);
  const drag = useDragReorder({ ids, axis: "y", containerRef: ref, group: "essai", onReorder });
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

  it("fait glisser les voisins et non la case tenue, le temps du geste", () => {
    const { getByTestId } = render(<Reorderable ids={["a", "b", "c"]} onReorder={vi.fn()} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 56 });

    expect(getByTestId("item-a").style.transition).toBe("none");
    expect(getByTestId("item-b").style.transition).toBe("transform var(--ease-smooth)");
  });

  it("retire la durée du mouvement au relâchement", () => {
    const { getByTestId } = render(<Reorderable ids={["a", "b", "c"]} onReorder={vi.fn()} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 56 });
    fireEvent.pointerUp(window);

    expect(getByTestId("item-b").style.transition).toBe("");
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

  /* Une liste de deux n'est faite que d'extrémités : si les bords sont
     inatteignables, plus rien ne se réordonne du tout. */
  it("fait passer la première case sous la seconde", () => {
    const onReorder = vi.fn();
    const { getByTestId } = render(<Reorderable ids={["a", "b"]} onReorder={onReorder} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 55 });
    fireEvent.pointerUp(window);

    expect(onReorder).toHaveBeenCalledWith(["b", "a"], 0, 1);
  });

  it("fait passer la seconde case au-dessus de la première", () => {
    const onReorder = vi.fn();
    const { getByTestId } = render(<Reorderable ids={["a", "b"]} onReorder={onReorder} />);

    grab(getByTestId("item-b"), 50);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 5 });
    fireEvent.pointerUp(window);

    expect(onReorder).toHaveBeenCalledWith(["b", "a"], 1, 0);
  });

  it("atteint la dernière place d'une liste de trois", () => {
    const onReorder = vi.fn();
    const { getByTestId } = render(<Reorderable ids={["a", "b", "c"]} onReorder={onReorder} />);

    grab(getByTestId("item-a"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 200 });
    fireEvent.pointerUp(window);

    expect(onReorder).toHaveBeenCalledWith(["b", "c", "a"], 0, 2);
  });

  it("ignore les cases d'une autre liste posées dans le même conteneur", () => {
    const onReorder = vi.fn();
    function TwoGroups() {
      const ref = useRef<HTMLDivElement>(null);
      const mine = useDragReorder({ ids: ["p", "q"], axis: "y", containerRef: ref, group: "mienne", onReorder });
      const other = useDragReorder({ ids: ["r"], axis: "y", containerRef: ref, group: "autre", onReorder: vi.fn() });
      return (
        <div ref={ref}>
          <div data-testid="item-p" {...mine.itemProps("p")} {...mine.handleProps("p")} />
          <div data-testid="item-r" {...other.itemProps("r")} {...other.handleProps("r")} />
          <div data-testid="item-q" {...mine.itemProps("q")} {...mine.handleProps("q")} />
        </div>
      );
    }
    const { getByTestId } = render(<TwoGroups />);

    /* Assez pour franchir le milieu de « r », pas celui de « q ». Si les deux
       listes se mélangeaient, ce geste rangerait quelque chose. */
    grab(getByTestId("item-p"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 60 });
    fireEvent.pointerUp(window);

    expect(onReorder).not.toHaveBeenCalled();
  });

  it("range la liste imbriquée quand le milieu de sa propre voisine est franchi", () => {
    const onReorder = vi.fn();
    function TwoGroups() {
      const ref = useRef<HTMLDivElement>(null);
      const mine = useDragReorder({ ids: ["p", "q"], axis: "y", containerRef: ref, group: "mienne", onReorder });
      const other = useDragReorder({ ids: ["r"], axis: "y", containerRef: ref, group: "autre", onReorder: vi.fn() });
      return (
        <div ref={ref}>
          <div data-testid="item-p" {...mine.itemProps("p")} {...mine.handleProps("p")} />
          <div data-testid="item-r" {...other.itemProps("r")} {...other.handleProps("r")} />
          <div data-testid="item-q" {...mine.itemProps("q")} {...mine.handleProps("q")} />
        </div>
      );
    }
    const { getByTestId } = render(<TwoGroups />);

    grab(getByTestId("item-p"), 10);
    fireEvent.pointerMove(window, { clientX: 0, clientY: 110 });
    fireEvent.pointerUp(window);

    expect(onReorder).toHaveBeenCalledWith(["q", "p"], 0, 1);
  });

  it("abandonne le geste sans rien enregistrer quand on l'annule", () => {
    const onReorder = vi.fn();
    function Cancellable() {
      const ref = useRef<HTMLDivElement>(null);
      const drag = useDragReorder({ ids: ["a", "b", "c"], axis: "y", containerRef: ref, group: "essai", onReorder });
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
