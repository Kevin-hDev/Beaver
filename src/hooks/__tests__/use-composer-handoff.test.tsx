import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render } from "@testing-library/react";
import { useRef } from "react";
import { useComposerHandoff } from "../use-composer-handoff";
import { noteComposerPosition, takeComposerPosition } from "@/lib/composer-handoff";

/* La bulle du champ dans la conversation : le haut de sa place définitive. */
const CHAT_BUBBLE_TOP = 620;

function Column({ ready }: { ready: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  useComposerHandoff(ref, ready);
  return (
    <div className="chat-input-column" ref={ref}>
      <div className="chat-input-bubble" />
    </div>
  );
}

function column(container: HTMLElement): HTMLElement {
  return container.querySelector(".chat-input-column") as HTMLElement;
}

/* Le point de départ n'existe qu'un instant : posé, lu par le navigateur, puis
   relâché dans le même souffle. C'est cette lecture qu'on intercepte — sans
   elle, le test passerait quelle que soit la distance calculée. */
let startPoint = "";

const realComputedStyle = window.getComputedStyle.bind(window);

beforeEach(() => {
  startPoint = "";
  /* jsdom ne calcule aucune feuille de style : sans ce relais, la transition
     dure zéro et le champ se poserait sans jamais glisser. */
  vi.spyOn(window, "getComputedStyle").mockImplementation((element) => {
    const base = realComputedStyle(element);
    return { ...base, transitionDuration: "340ms" };
  });
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
    this: HTMLElement,
  ) {
    if (this.classList.contains("chat-input-column")) startPoint = this.style.transform;
    const top = this.classList.contains("chat-input-bubble") ? CHAT_BUBBLE_TOP : 0;
    return { top, bottom: top, left: 0, right: 0, width: 0, height: 0, x: 0, y: top, toJSON: () => ({}) };
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  takeComposerPosition();
});

describe("arrivée du champ depuis l'écran d'accueil", () => {
  it("part de la position notée et rejoint la sienne", () => {
    noteComposerPosition(320);
    const { container, rerender } = render(<Column ready={false} />);

    expect(column(container).style.transform).toBe("");

    rerender(<Column ready />);
    const node = column(container);

    /* 320 - 620 : le champ part trois cents pixels plus haut, puis est relâché. */
    expect(startPoint).toBe("translateY(-300px)");
    expect(node.classList.contains("chat-composer-arriving")).toBe(true);
    expect(node.style.transform).toBe("");

    act(() => {
      node.dispatchEvent(new Event("transitionend"));
    });

    expect(node.classList.contains("chat-composer-arriving")).toBe(false);
  });

  /* Depuis que la conversation se montre dès son premier rendu, c'est ce
     chemin-là qui sert : le champ doit partir du bon endroit sans attendre
     aucun changement d'état. */
  it("part du bon endroit dès le premier rendu", () => {
    noteComposerPosition(320);

    const { container } = render(<Column ready />);

    expect(startPoint).toBe("translateY(-300px)");
    expect(column(container).classList.contains("chat-composer-arriving")).toBe(true);
  });

  it("ne bouge pas quand aucune position n'a été notée", () => {
    const { container, rerender } = render(<Column ready={false} />);

    rerender(<Column ready />);

    expect(column(container).classList.contains("chat-composer-arriving")).toBe(false);
  });

  /* Tant que la conversation est transparente, le glissement se jouerait sans
     témoin et le champ paraîtrait surgir à sa place. */
  it("attend que la conversation soit visible", () => {
    noteComposerPosition(320);
    const { container } = render(<Column ready={false} />);

    expect(column(container).classList.contains("chat-composer-arriving")).toBe(false);
    expect(takeComposerPosition()).toBe(320);
  });

  it("consomme la position pour que la conversation suivante ne bouge pas", () => {
    noteComposerPosition(320);
    const { rerender } = render(<Column ready={false} />);
    rerender(<Column ready />);

    expect(takeComposerPosition()).toBeNull();
  });
});
