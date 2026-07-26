import { describe, expect, it } from "vitest";
import { selectReleaseNotes } from "../update-release-notes";

describe("selectReleaseNotes", () => {
  it("sélectionne les notes dans la langue active", () => {
    const notes = selectReleaseNotes(
      {
        en: ["English note."],
        fr: ["Note française."],
      },
      "fr",
    );

    expect(notes).toEqual(["Note française."]);
  });

  it("revient à l'anglais si la langue active manque", () => {
    const notes = selectReleaseNotes({ en: ["English fallback."] }, "de");

    expect(notes).toEqual(["English fallback."]);
  });

  it("limite les notes sans tronquer les phrases", () => {
    const notes = selectReleaseNotes({
      en: Array.from({ length: 10 }, (_, i) => `Item ${i}.`),
    });

    expect(notes).toHaveLength(6);
    expect(notes.join("\n")).not.toContain("…");
  });

  it("ignore les lignes trop longues", () => {
    expect(selectReleaseNotes({ en: [`${"x".repeat(181)}.`] })).toEqual([]);
  });
});
