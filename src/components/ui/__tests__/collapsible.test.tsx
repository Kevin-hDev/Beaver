import { afterEach, describe, expect, it } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { useLayoutEffect, useState } from "react";
import { Collapsible } from "../collapsible";

afterEach(cleanup);

function Harness({ unmountWhenClosed = false }: { unmountWhenClosed?: boolean }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button type="button" aria-expanded={open} onClick={() => setOpen((value) => !value)}>
        toggle
      </button>
      <Collapsible open={open} unmountWhenClosed={unmountWhenClosed}>
        <p>contenu</p>
      </Collapsible>
    </>
  );
}

describe("Collapsible", () => {
  it("garde le contenu monté par défaut, même fermé", () => {
    const { container, queryByText } = render(
      <Collapsible open={false}>
        <p>contenu</p>
      </Collapsible>,
    );

    expect(queryByText("contenu")).not.toBeNull();
    expect(container.querySelector<HTMLElement>(".cps-region")?.style.height).toBe("0px");
  });

  it("retire le contenu du DOM quand on le lui demande", () => {
    const { queryByText } = render(
      <Collapsible open={false} unmountWhenClosed>
        <p>contenu</p>
      </Collapsible>,
    );

    expect(queryByText("contenu")).toBeNull();
  });

  it("monte le contenu dès l'ouverture, sans attendre un rendu supplémentaire", () => {
    const { getByRole, queryByText } = render(<Harness unmountWhenClosed />);

    expect(queryByText("contenu")).toBeNull();
    fireEvent.click(getByRole("button"));
    expect(queryByText("contenu")).not.toBeNull();
  });

  it("libère la hauteur et l'overflow une fois ouvert", () => {
    const { container } = render(
      <Collapsible open>
        <p>contenu</p>
      </Collapsible>,
    );

    const region = container.querySelector<HTMLElement>(".cps-region");
    expect(region?.style.height).toBe("auto");
    // Relâché pour ne pas trancher les couches flottantes du contenu.
    expect(region?.style.overflow).toBe("visible");
  });

  it("revient à l'état fermé et démonte quand la transition ne peut pas se jouer", () => {
    const { getByRole, queryByText } = render(<Harness unmountWhenClosed />);
    const toggle = getByRole("button");

    fireEvent.click(toggle);
    expect(queryByText("contenu")).not.toBeNull();

    fireEvent.click(toggle);
    expect(queryByText("contenu")).toBeNull();
  });

  it("annonce son ouverture dès le rendu, avant tout effet", () => {
    /* Les effets des enfants s'exécutent avant celui du parent : un contenu
       qui se mesure lui-même — un graphe, par exemple — se dessinait dans une
       région encore haute de zéro, et gardait cette taille. L'état de repos
       doit donc être lisible dans le balisage, pas seulement dans un effet.
       Défaut relevé par Kevin sur le graphe principal de forecast. */
    let hauteurAuMontage: string | null = null;
    function Sonde() {
      useLayoutEffect(() => {
        const region = document.querySelector(".cps-region");
        hauteurAuMontage = region?.getAttribute("data-open") ?? null;
      }, []);
      return <p>contenu</p>;
    }
    render(
      <Collapsible open>
        <Sonde />
      </Collapsible>,
    );
    expect(hauteurAuMontage).toBe("true");
  });

  it("ferme la région par le balisage et non par un effet", () => {
    const { container } = render(
      <Collapsible open={false}>
        <p>contenu</p>
      </Collapsible>,
    );
    expect(container.querySelector(".cps-region")?.getAttribute("data-open")).toBe("false");
  });

  it("ignore une fin de transition remontée par un enfant", () => {
    const { container, getByRole, queryByText } = render(<Harness unmountWhenClosed />);
    fireEvent.click(getByRole("button"));

    const inner = container.querySelector(".cps-inner");
    expect(inner).not.toBeNull();
    fireEvent.transitionEnd(inner!, { propertyName: "height" });

    expect(queryByText("contenu")).not.toBeNull();
  });
});
