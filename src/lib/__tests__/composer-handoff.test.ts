import { afterEach, describe, expect, it, vi } from "vitest";
import { hasComposerPosition, noteComposerPosition, takeComposerPosition } from "../composer-handoff";

afterEach(() => {
  vi.useRealTimers();
  takeComposerPosition();
});

describe("position transmise à la conversation", () => {
  it("rend la position notée", () => {
    noteComposerPosition(412);

    expect(takeComposerPosition()).toBe(412);
  });

  /* Sans effacement, une conversation ouverte ensuite reprendrait la position
     d'un envoi déjà consommé et son champ sauterait sans raison. */
  it("s'efface à la lecture", () => {
    noteComposerPosition(412);
    takeComposerPosition();

    expect(takeComposerPosition()).toBeNull();
  });

  it("rend null quand rien n'a été noté", () => {
    expect(takeComposerPosition()).toBeNull();
  });

  /* La conversation doit savoir qu'une reprise est en cours dès son premier
     rendu, bien avant d'être en mesure de mesurer quoi que ce soit. */
  it("s'annonce sans se consommer", () => {
    noteComposerPosition(412);

    expect(hasComposerPosition()).toBe(true);
    expect(hasComposerPosition()).toBe(true);
    expect(takeComposerPosition()).toBe(412);
    expect(hasComposerPosition()).toBe(false);
  });

  it("ne s'annonce pas quand la position est périmée", () => {
    vi.useFakeTimers();
    noteComposerPosition(412);
    vi.advanceTimersByTime(2100);

    expect(hasComposerPosition()).toBe(false);
  });

  /* Un envoi abandonné laisse une position derrière lui. Elle ne doit pas
     déplacer le champ d'une conversation ouverte bien plus tard. */
  it("périme une position trop ancienne", () => {
    vi.useFakeTimers();
    noteComposerPosition(412);
    vi.advanceTimersByTime(2100);

    expect(takeComposerPosition()).toBeNull();
  });

  it("garde une position encore fraîche", () => {
    vi.useFakeTimers();
    noteComposerPosition(412);
    vi.advanceTimersByTime(500);

    expect(takeComposerPosition()).toBe(412);
  });
});
