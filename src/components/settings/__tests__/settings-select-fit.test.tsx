import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { SettingsSelect } from "../settings-select";

afterEach(cleanup);

const MODELS = [
  { value: "chronos-2", label: "Chronos-2" },
  { value: "chronos-bolt-small", label: "Chronos-Bolt Small" },
  { value: "moirai", label: "MOIRAI 2.0 R Small" },
];

/* La largeur du bouton vient du mesureur, qui porte tous les libellés : c'est
   ce qui empêche un nom de modèle d'être tronqué et garde la largeur stable
   d'une sélection à l'autre. */
describe("SettingsSelect, largeur calée sur le libellé le plus long", () => {
  it("rend le mesureur avec tous les libellés quand la mise à la largeur est demandée", () => {
    const { container } = render(
      <SettingsSelect options={MODELS} value="chronos-2" onChange={vi.fn()} fitLongestOption />,
    );

    const sizer = container.querySelector(".ss-trigger-sizer");
    expect(sizer).not.toBeNull();
    expect([...sizer!.children].map((node) => node.textContent)).toEqual(
      MODELS.map((model) => model.label),
    );
    expect(container.querySelector(".ss-wrap")?.classList.contains("ss-fit")).toBe(true);
  });

  it("garde la largeur commune par défaut", () => {
    const { container } = render(
      <SettingsSelect options={MODELS} value="chronos-2" onChange={vi.fn()} />,
    );

    expect(container.querySelector(".ss-trigger-sizer")).toBeNull();
    expect(container.querySelector(".ss-wrap")?.classList.contains("ss-fit")).toBe(false);
  });
});
