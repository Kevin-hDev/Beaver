import { describe, expect, it } from "vitest";
import type { ToolActivityRecord } from "@/types/agent";
import { extractToolPath, inferSavedToolPaths, isFileTool, lastSavedToolPath } from "./tool-file-path";

function makeTool(name: string, summary: string): ToolActivityRecord {
  return { name, summary };
}

describe("isFileTool", () => {
  it("retourne true pour chaque outil fichier connu", () => {
    const FILE_TOOLS = [
      "read_file", "write_file", "edit_file", "read_spreadsheet",
      "read_document", "read_image", "write_spreadsheet", "write_document", "process_image",
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

describe("extractToolPath", () => {
  it("trouve la clé 'path' en priorité", () => {
    const result = extractToolPath({ path: "/home/user/file.ts", file_path: "/other/path.ts" });
    expect(result).toBe("/home/user/file.ts");
  });

  it("essaie les clés alternatives dans l'ordre", () => {
    expect(extractToolPath({ file_path: "/a/b.ts" })).toBe("/a/b.ts");
    expect(extractToolPath({ filepath: "/c/d.ts" })).toBe("/c/d.ts");
    expect(extractToolPath({ target_path: "/e/f.ts" })).toBe("/e/f.ts");
  });

  it("retourne une chaîne vide si aucune clé reconnue", () => {
    expect(extractToolPath({ url: "/some/url", content: "data" })).toBe("");
    expect(extractToolPath({})).toBe("");
  });

  it("ignore les valeurs vides ou composées uniquement d'espaces", () => {
    expect(extractToolPath({ path: "   ", file_path: "/valid/path.ts" })).toBe("/valid/path.ts");
    expect(extractToolPath({ path: "" })).toBe("");
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

describe("lastSavedToolPath", () => {
  it("retourne le dernier chemin d'un outil fichier avec summary", () => {
    const tools: ToolActivityRecord[] = [
      makeTool("read_file", "/first.ts"),
      makeTool("bash", "/not-a-file-tool"),
      makeTool("write_file", "/second.ts"),
    ];
    expect(lastSavedToolPath(tools)).toBe("/second.ts");
  });

  it("retourne initialPath si aucun tool file avec summary", () => {
    const tools: ToolActivityRecord[] = [
      makeTool("bash", "/ignored"),
      makeTool("read_file", ""),
    ];
    expect(lastSavedToolPath(tools, "/default.ts")).toBe("/default.ts");
  });

  it("retourne une chaîne vide si liste vide et pas d'initialPath", () => {
    expect(lastSavedToolPath([])).toBe("");
  });

  it("retourne initialPath si aucun tool file dans la liste", () => {
    const tools: ToolActivityRecord[] = [
      makeTool("bash", "/chemin/quelconque"),
      makeTool("grep", "/autre/chemin"),
    ];
    expect(lastSavedToolPath(tools, "/mon-chemin-initial.ts")).toBe("/mon-chemin-initial.ts");
  });
});

describe("extractToolPath — cas limites supplémentaires", () => {
  it("chemin Windows avec backslashes est retourné tel quel", () => {
    const result = extractToolPath({ path: "C:\\Users\\Kevin\\project\\main.ts" });
    expect(result).toBe("C:\\Users\\Kevin\\project\\main.ts");
  });

  it("valeur non-string (number) pour path → ignorée, retourne chaîne vide", () => {
    const result = extractToolPath({ path: 42 });
    expect(result).toBe("");
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
