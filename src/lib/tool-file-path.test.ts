import { describe, expect, it } from "vitest";
import type { ToolActivityRecord } from "@/types/agent";
import { inferSavedToolPaths, isFileTool } from "./tool-file-path";

function makeTool(name: string, summary: string): ToolActivityRecord {
  return { name, summary };
}

describe("isFileTool", () => {
  it("retourne true pour chaque outil fichier connu", () => {
    const FILE_TOOLS = [
      "read_file", "write_file", "edit_file", "read_spreadsheet",
      "read_document", "write_spreadsheet", "write_document", "transform_image",
    ];
    for (const tool of FILE_TOOLS) {
      expect(isFileTool(tool), `attendu true pour "${tool}"`).toBe(true);
    }
  });

  it("retourne false pour les outils non-fichier", () => {
    expect(isFileTool("bash")).toBe(false);
    expect(isFileTool("grep")).toBe(false);
    expect(isFileTool("web_search")).toBe(false);
    expect(isFileTool("")).toBe(false);
  });
});

describe("inferSavedToolPaths", () => {
  it("propage le dernier path aux tools file sans summary", () => {
    const tools: ToolActivityRecord[] = [
      makeTool("read_file", "/project/main.ts"),
      makeTool("read_file", ""),
      makeTool("read_file", ""),
    ];
    const result = inferSavedToolPaths(tools);
    expect(result[1].summary).toBe("/project/main.ts");
    expect(result[2].summary).toBe("/project/main.ts");
  });

  it("utilise initialPath comme fallback pour le premier tool sans summary", () => {
    const tools: ToolActivityRecord[] = [makeTool("write_file", "")];
    const result = inferSavedToolPaths(tools, "/default/path.ts");
    expect(result[0].summary).toBe("/default/path.ts");
  });

  it("ne modifie pas les outils non-fichier", () => {
    const tools: ToolActivityRecord[] = [
      makeTool("read_file", "/file.ts"),
      makeTool("bash", ""),
      makeTool("read_file", ""),
    ];
    const result = inferSavedToolPaths(tools);
    expect(result[1].summary).toBe("");
    expect(result[2].summary).toBe("/file.ts");
  });

  it("retourne la même référence d'objet si le summary n'a pas changé", () => {
    const tool = makeTool("read_file", "/file.ts");
    const result = inferSavedToolPaths([tool]);
    expect(result[0]).toBe(tool);
  });
});

describe("inferSavedToolPaths — cas limites supplémentaires", () => {
  it("outil non-file (bash) ne propage pas son summary comme path", () => {
    const tools: ToolActivityRecord[] = [
      makeTool("bash", "/chemin/bash"),
      makeTool("read_file", ""),
    ];
    const result = inferSavedToolPaths(tools);
    // bash ne compte pas comme path, read_file sans summary reste ""
    expect(result[1].summary).toBe("");
  });
});
