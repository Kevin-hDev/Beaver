import { describe, expect, it } from "vitest";
import { buildToolTimelineBlocks } from "../message-tool-blocks";

describe("buildToolTimelineBlocks", () => {
  it("fusionne les segments tools-only avec la dernière réflexion", () => {
    const blocks = buildToolTimelineBlocks([
      { thinking: "réflexion 1", content: "", tools: ["bash"] },
      { thinking: "", content: "", tools: ["write_file"] },
      { thinking: "réflexion 2", content: "", tools: ["read_file"] },
    ]);

    expect(blocks).toEqual([
      { thinking: "réflexion 1", content: "", tools: ["bash", "write_file"], isCurrent: false },
      { thinking: "réflexion 2", content: "", tools: ["read_file"], isCurrent: false },
    ]);
  });

  it("ne marque pas une ancienne réflexion comme active si seul le tool courant est actif", () => {
    const blocks = buildToolTimelineBlocks([
      { thinking: "réflexion", content: "", tools: ["bash"] },
      { thinking: "", content: "", tools: ["edit_file"], isCurrent: true },
    ]);

    expect(blocks).toEqual([
      { thinking: "réflexion", content: "", tools: ["bash", "edit_file"], isCurrent: false },
    ]);
  });
});
