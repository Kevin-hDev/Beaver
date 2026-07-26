import { describe, expect, it } from "vitest";
import { findSkillTokenRanges } from "@/components/agent-local/skill-chip-ranges";

describe("findSkillTokenRanges", () => {
  it("détecte un skill simple avec bornes correctes", () => {
    const text = "Charge /context7-docs maintenant";
    const ranges = findSkillTokenRanges(text, ["context7-docs"], []);
    expect(ranges).toHaveLength(1);
    expect(ranges[0]).toMatchObject({
      name: "context7-docs",
      raw: "/context7-docs",
      source: "skill",
      from: 7,
      to: 21,
    });
    expect(text.slice(ranges[0].from, ranges[0].to)).toBe("/context7-docs");
  });

  it("détecte plusieurs skills dans le même texte", () => {
    const ranges = findSkillTokenRanges(
      "A /context7-docs et /frontend-design",
      ["context7-docs", "frontend-design"],
      [],
    );
    expect(ranges.map((r) => r.name)).toEqual(["context7-docs", "frontend-design"]);
  });

  it("marque les built-ins avec source 'built-in'", () => {
    const ranges = findSkillTokenRanges("Compresse /compress", [], ["compress"]);
    expect(ranges).toHaveLength(1);
    expect(ranges[0].source).toBe("built-in");
    expect(ranges[0].name).toBe("compress");
  });

  it("un built-in prime sur un skill de même nom", () => {
    // Si l'utilisateur a un skill nommé "compress" ET un built-in "compress",
    // le built-in gagne pour le rendu (source = "built-in").
    const ranges = findSkillTokenRanges("/compress", ["compress"], ["compress"]);
    expect(ranges).toHaveLength(1);
    expect(ranges[0].source).toBe("built-in");
  });

  it("ignore les tokens collés à un mot", () => {
    expect(findSkillTokenRanges("foo/context7-docs", ["context7-docs"], [])).toEqual([]);
  });

  it("respecte la ponctuation comme borne de fin", () => {
    const text = "Skill: /context7-docs.";
    const ranges = findSkillTokenRanges(text, ["context7-docs"], []);
    expect(ranges).toHaveLength(1);
    expect(text.slice(ranges[0].from, ranges[0].to)).toBe("/context7-docs");
  });

  it("retourne un tableau vide quand aucun nom n'est fourni", () => {
    expect(findSkillTokenRanges("texte /random", [], [])).toEqual([]);
  });
});
