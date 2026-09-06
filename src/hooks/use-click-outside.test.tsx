import { useRef } from "react";
import { createPortal } from "react-dom";
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useClickOutside } from "./use-click-outside";

afterEach(cleanup);

function Probe({ onOutside, withFloating }: { onOutside: () => void; withFloating: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  const floatingRef = useRef<HTMLDivElement>(null);
  useClickOutside(ref, onOutside, withFloating ? floatingRef : undefined);
  return (
    <div ref={ref}>
      <button type="button">dedans</button>
      {createPortal(<div ref={floatingRef}><button type="button">flottant</button></div>, document.body)}
    </div>
  );
}

describe("useClickOutside", () => {
  it("signale un clic hors du conteneur", () => {
    const onOutside = vi.fn();
    render(<Probe onOutside={onOutside} withFloating={false} />);
    fireEvent.mouseDown(document.body);
    expect(onOutside).toHaveBeenCalledTimes(1);
  });

  it("ignore un clic dans le conteneur", () => {
    const onOutside = vi.fn();
    render(<Probe onOutside={onOutside} withFloating={false} />);
    fireEvent.mouseDown(screen.getByText("dedans"));
    expect(onOutside).not.toHaveBeenCalled();
  });

  /* Sans le second repère, une couche sortie par un portail compte comme
     « dehors » : elle se referme au premier clic, avant que l'action ne parte. */
  it("ignore un clic dans la couche portée ailleurs quand elle est déclarée", () => {
    const onOutside = vi.fn();
    render(<Probe onOutside={onOutside} withFloating />);
    fireEvent.mouseDown(screen.getByText("flottant"));
    expect(onOutside).not.toHaveBeenCalled();
  });

  it("signale ce même clic quand la couche n'est pas déclarée", () => {
    const onOutside = vi.fn();
    render(<Probe onOutside={onOutside} withFloating={false} />);
    fireEvent.mouseDown(screen.getByText("flottant"));
    expect(onOutside).toHaveBeenCalledTimes(1);
  });
});
