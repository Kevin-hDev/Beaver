import { describe, expect, it } from "vitest";
import { getMascotAnimation, spritePosition } from "./mascot-assets";
import {
  mascotFrameDuration,
  nextMascotFrame,
  selectMascotAnimation,
} from "./use-mascot-animation";

describe("mascot sprite playback", () => {
  it("boucle uniquement les animations prévues", () => {
    expect(nextMascotFrame(5, 6, true)).toBe(0);
    expect(nextMascotFrame(5, 6, false)).toBe(5);
  });

  it("borne le nombre d'images à la largeur de la planche", () => {
    const animation = getMascotAnimation("work-laptop");
    expect(animation.frames).toBe(8);
    expect(animation.row).toBe(11);
  });

  it("utilise les planches et rythmes propres à Circuit", () => {
    const work = getMascotAnimation("work-laptop", "circuit");
    const thinking = getMascotAnimation("thinking", "circuit");
    const held = getMascotAnimation("held", "circuit");

    expect(work).toMatchObject({ row: 0, frames: 6, columns: 6, rows: 6 });
    expect(thinking).toMatchObject({ row: 2, frames: 6, loopPauseMs: 3000 });
    expect(held).toMatchObject({ row: 4, frames: 6, frameDurationMs: 120 });
  });

  it("utilise les planches standard et les poses avancées de Kova", () => {
    const idle = getMascotAnimation("idle", "kova");
    const work = getMascotAnimation("work-laptop", "kova");
    const exploration = getMascotAnimation("explore-book", "kova");
    const success = getMascotAnimation("success", "kova");

    expect(idle).toMatchObject({
      row: 0,
      startFrame: 0,
      frames: 6,
      columns: 8,
      rows: 11,
    });
    expect(work).toMatchObject({ row: 0, startFrame: 0, frames: 3, rows: 3 });
    expect(exploration).toMatchObject({ row: 0, startFrame: 5, frames: 3 });
    expect(success).toMatchObject({ row: 0, startFrame: 3, frames: 1 });
  });

  it("utilise les planches standard et les poses avancées de Nival", () => {
    const idle = getMascotAnimation("idle", "nival");
    const work = getMascotAnimation("work-laptop", "nival");
    const exploration = getMascotAnimation("explore-book", "nival");
    const success = getMascotAnimation("success", "nival");
    const held = getMascotAnimation("held", "nival");

    expect(idle).toMatchObject({
      row: 0,
      startFrame: 0,
      frames: 6,
      columns: 8,
      rows: 11,
    });
    expect(work).toMatchObject({ row: 0, startFrame: 0, frames: 3, rows: 3 });
    expect(exploration).toMatchObject({ row: 0, startFrame: 5, frames: 3 });
    expect(success).toMatchObject({ row: 0, startFrame: 3, frames: 1 });
    expect(held).toMatchObject({ row: 1, startFrame: 6, frames: 1 });
  });

  it("positionne correctement les coins de la planche", () => {
    expect(spritePosition(0, 0)).toBe("0% 0%");
    expect(spritePosition(7, 18)).toBe("100% 100%");
    expect(spritePosition(7, 2, 8, 3)).toBe("100% 100%");
  });

  it("conserve le repos tant qu'aucun état réel ne le remplace", () => {
    expect(selectMascotAnimation("idle", null)).toBe("idle");
    expect(selectMascotAnimation("work-laptop", null)).toBe("work-laptop");
    expect(selectMascotAnimation("thinking", "grabbed")).toBe("grabbed");
  });

  it("applique les pauses communes à chaque mascotte", () => {
    for (const mascotId of ["cl-go-beaver", "circuit", "kova", "nival"] as const) {
      const idle = getMascotAnimation("idle", mascotId);
      const waiting = getMascotAnimation("waiting", mascotId);
      const thinking = getMascotAnimation("thinking", mascotId);
      const exploration = getMascotAnimation("explore-book", mascotId);
      const work = getMascotAnimation("work-laptop", mascotId);

      expect(mascotFrameDuration(idle, idle.frames - 1)).toBe(4500);
      expect(mascotFrameDuration(waiting, waiting.frames - 1)).toBe(3500);
      expect(mascotFrameDuration(thinking, thinking.frames - 1)).toBe(3000);
      expect(mascotFrameDuration(exploration, exploration.frames - 1)).toBe(3000);
      expect(mascotFrameDuration(work, work.frames - 1)).toBe(2500);
    }
  });
});
