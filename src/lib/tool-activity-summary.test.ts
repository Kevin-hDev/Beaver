import { describe, expect, it } from "vitest";
import { groupIcon, groupToolActivities } from "./tool-activity-summary";

function tool(name: string, result = "ok") {
  return { name, summary: name, result };
}

describe("groupToolActivities", () => {
  it("groupe les lectures, listes et recherches en exploration", () => {
    const groups = groupToolActivities([
      ...Array.from({ length: 8 }, () => tool("read_file")),
      tool("list_dir"),
      tool("grep"),
    ]);

    expect(groups).toHaveLength(1);
    expect(groups[0].kind).toBe("exploration");
    expect(groups[0].counts.files).toBe(8);
    expect(groups[0].counts.lists).toBe(1);
    expect(groups[0].counts.searches).toBe(1);
  });

  it("groupe les écritures et éditions en modifications", () => {
    const groups = groupToolActivities([tool("write_file"), tool("edit_file")]);

    expect(groups).toHaveLength(1);
    expect(groups[0].kind).toBe("modification");
    expect(groups[0].counts.writes).toBe(1);
    expect(groups[0].counts.edits).toBe(1);
  });

  it("groupe les recherches web et fetch web ensemble", () => {
    const groups = groupToolActivities([tool("web_search"), tool("web_fetch")]);

    expect(groups).toHaveLength(1);
    expect(groups[0].kind).toBe("web");
    expect(groups[0].counts.webSearches).toBe(1);
    expect(groups[0].counts.webFetches).toBe(1);
  });

  it("groupe bash en commandes", () => {
    const groups = groupToolActivities([tool("bash")]);

    expect(groups).toHaveLength(1);
    expect(groups[0].kind).toBe("command");
    expect(groups[0].counts.commands).toBe(1);
  });

  it("met les tools inconnus dans autres", () => {
    const groups = groupToolActivities([tool("load_skill")]);

    expect(groups).toHaveLength(1);
    expect(groups[0].kind).toBe("other");
    expect(groups[0].counts.otherActions).toBe(1);
  });

  it("marque un groupe en cours tant qu'un tool n'a pas de résultat", () => {
    const groups = groupToolActivities([{ name: "bash", summary: "sleep" }]);

    expect(groups[0].isPending).toBe(true);
    expect(groups[0].hasError).toBe(false);
  });
});

describe("groupIcon", () => {
  it("associe la bonne icône Phosphor à chaque type de groupe", () => {
    expect(groupIcon("exploration")).toBe("Compass");
    expect(groupIcon("modification")).toBe("PencilSimple");
    expect(groupIcon("command")).toBe("TerminalWindow");
    expect(groupIcon("web")).toBe("Globe");
    expect(groupIcon("git")).toBe("GitBranch");
    expect(groupIcon("other")).toBe("Wrench");
  });
});
