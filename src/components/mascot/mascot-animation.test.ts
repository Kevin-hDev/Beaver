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

  it("positionne correctement les coins de la planche", () => {
    expect(spritePosition(0, 0)).toBe("0% 0%");
    expect(spritePosition(7, 18)).toBe("100% 100%");
  });

  it("conserve le repos tant qu'aucun état réel ne le remplace", () => {
    expect(selectMascotAnimation("idle", null)).toBe("idle");
    expect(selectMascotAnimation("work-laptop", null)).toBe("work-laptop");
    expect(selectMascotAnimation("thinking", "grabbed")).toBe("grabbed");
  });

  it("laisse une pause adaptée après chaque animation en boucle", () => {
    const idle = getMascotAnimation("idle");
    const waiting = getMascotAnimation("waiting");
    const thinking = getMascotAnimation("thinking");
    const exploration = getMascotAnimation("explore-book");
    const work = getMascotAnimation("work-laptop");

    expect(mascotFrameDuration(idle, 0)).toBe(260);
    expect(mascotFrameDuration(idle, idle.frames - 1)).toBe(3500);
    expect(mascotFrameDuration(waiting, waiting.frames - 1)).toBe(2500);
    expect(mascotFrameDuration(thinking, thinking.frames - 1)).toBe(1500);
    expect(mascotFrameDuration(exploration, exploration.frames - 1)).toBe(2000);
    expect(mascotFrameDuration(work, work.frames - 1)).toBe(1500);
  });
});
