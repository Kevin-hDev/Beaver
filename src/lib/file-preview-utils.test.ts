import { describe, expect, it } from "vitest";
import {
  collectMessageFileOperations,
  collectFileOperations,
  countLines,
  fileNameFromPath,
  shortPath,
} from "./file-preview-utils";
import type { AgentMessage, ToolActivityRecord } from "@/types/agent";

describe("fileNameFromPath", () => {
  it("extrait le nom de fichier depuis un chemin Unix", () => {
    expect(fileNameFromPath("/home/user/project/main.ts")).toBe("main.ts");
    expect(fileNameFromPath("/src/lib/utils.ts")).toBe("utils.ts");
  });

  it("extrait le nom de fichier depuis un chemin Windows (backslash)", () => {
    expect(fileNameFromPath("C:\\Users\\Kevin\\project\\index.ts")).toBe("index.ts");
    expect(fileNameFromPath("C:\\src\\main.rs")).toBe("main.rs");
  });

  it("retourne le chemin tel quel si pas de séparateur", () => {
    expect(fileNameFromPath("fichier.ts")).toBe("fichier.ts");
    expect(fileNameFromPath("")).toBe("");
  });

  it("gère les chemins avec séparateurs mixtes", () => {
    expect(fileNameFromPath("/home/user\\project/file.ts")).toBe("file.ts");
  });
});

describe("shortPath", () => {
  it("strip le baseDir du début du chemin", () => {
    expect(shortPath("/home/user/project/src/main.ts", "/home/user/project")).toBe("src/main.ts");
  });

  it("normalise les backslashes Windows dans le chemin", () => {
    expect(shortPath("C:\\Users\\Kevin\\project\\src\\main.ts", "C:\\Users\\Kevin\\project")).toBe("src/main.ts");
  });

  it("retourne le chemin original si baseDir est absent", () => {
    expect(shortPath("/home/user/file.ts")).toBe("/home/user/file.ts");
    expect(shortPath("/home/user/file.ts", "")).toBe("/home/user/file.ts");
  });

  it("retourne le chemin original si le baseDir ne correspond pas", () => {
    expect(shortPath("/other/path/file.ts", "/home/user/project")).toBe("/other/path/file.ts");
  });

  it("tolère un baseDir avec slash final", () => {
    expect(shortPath("/home/user/project/file.ts", "/home/user/project/")).toBe("file.ts");
  });
});

describe("countLines", () => {
  it("compte les lignes séparées par \\n", () => {
    expect(countLines("ligne1\nligne2\nligne3")).toBe(3);
  });

  it("compte les lignes séparées par \\r\\n (Windows)", () => {
    expect(countLines("ligne1\r\nligne2\r\nligne3")).toBe(3);
  });

  it("retourne 1 pour une chaîne sans saut de ligne", () => {
    expect(countLines("une seule ligne")).toBe(1);
  });

  it("retourne 0 pour une valeur falsy", () => {
    expect(countLines(undefined)).toBe(0);
    expect(countLines("")).toBe(0);
  });
});

describe("fileNameFromPath — cas limites supplémentaires", () => {
  it("chemin vide retourne une chaîne vide", () => {
    expect(fileNameFromPath("")).toBe("");
  });
});

describe("shortPath — cas limites supplémentaires", () => {
  it("baseDir qui ne matche pas du tout retourne le chemin original", () => {
    expect(shortPath("/home/user/file.ts", "/srv/data")).toBe("/home/user/file.ts");
  });

  it("baseDir avec trailing slash est correctement géré", () => {
    expect(shortPath("/home/user/project/file.ts", "/home/user/project/")).toBe("file.ts");
  });
});

describe("countLines — cas limites supplémentaires", () => {
  it("retourne 0 pour undefined", () => {
    expect(countLines(undefined)).toBe(0);
  });

  it("retourne 0 pour chaîne vide (falsy)", () => {
    expect(countLines("")).toBe(0);
  });

  it("compte correctement les lignes avec retours chariot Windows (\\r\\n)", () => {
    expect(countLines("ligne1\r\nligne2\r\nligne3")).toBe(3);
  });
});

describe("collectFileOperations", () => {
  it("inclut un write_file live avant la fin du stream", () => {
    const operations = collectFileOperations([], {
      liveTools: [
        tool({ name: "write_file", summary: "/repo/live.ts", content: "const a = 1;", result: "ok" }),
      ],
    });

    expect(operations).toHaveLength(1);
    expect(operations[0]).toEqual(expect.objectContaining({
      path: "/repo/live.ts",
      name: "live.ts",
      type: "write",
      additions: 1,
      deletions: 0,
    }));
  });

  it("ne garde qu'une ligne par fichier et remonte le dernier fichier touché", () => {
    const messages = [
      message("m1", [tool({ name: "write_file", summary: "/repo/a.ts", content: "a" })]),
      message("m2", [tool({ name: "write_file", summary: "/repo/b.ts", content: "b" })]),
      message("m3", [tool({
        name: "edit_file",
        summary: "/repo/a.ts",
        old_text: "a\nb",
        new_text: "c",
      })]),
    ];

    const operations = collectFileOperations(messages);

    expect(operations.map((operation) => operation.path)).toEqual(["/repo/a.ts", "/repo/b.ts"]);
    expect(operations[0]).toEqual(expect.objectContaining({
      type: "edit",
      additions: 1,
      deletions: 2,
    }));
  });

  it("utilise le diff figé d'un remplacement complet", () => {
    const diff = {
      binary: false,
      truncated: false,
      hunks: [{
        old_start: 1, old_lines: 1, new_start: 1, new_lines: 1,
        lines: [
          { kind: "deleted" as const, content: "old", old_line: 1, new_line: null },
          { kind: "added" as const, content: "new", old_line: null, new_line: 1 },
        ],
      }],
    };
    const operations = collectFileOperations([
      message("m1", [tool({
        name: "write_file",
        is_error: false,
        file_changes: [{
          path: "/repo/a.ts", status: "modified", additions: 1, deletions: 1, diff,
        }],
      })]),
    ]);

    expect(operations[0]).toEqual(expect.objectContaining({
      path: "/repo/a.ts",
      recordedStatus: "modified",
      recordedDiff: diff,
      additions: 1,
      deletions: 1,
    }));
  });

  it("donne un identifiant différent au même fichier dans deux requêtes", () => {
    const first = collectMessageFileOperations(message("first", [
      tool({ summary: "/repo/a.ts", content: "first" }),
    ]));
    const second = collectMessageFileOperations(message("second", [
      tool({ summary: "/repo/a.ts", content: "second" }),
    ]));

    expect(first[0].id).not.toBe(second[0].id);
  });

  it("utilise le chemin résolu quand il est fourni par le backend", () => {
    const operations = collectFileOperations([
      message("m1", [tool({
        name: "edit_file",
        summary: "src/a.ts",
        resolved_path: "/repo/src/a.ts",
        old_text: "old",
        new_text: "new",
      })]),
    ]);

    expect(operations).toHaveLength(1);
    expect(operations[0]).toEqual(expect.objectContaining({
      path: "/repo/src/a.ts",
      name: "a.ts",
    }));
  });

  it("dédoublonne un ancien chemin relatif et une nouvelle opération absolue", () => {
    const operations = collectFileOperations([
      message("old", [tool({
        name: "edit_file",
        summary: "src/test_ui_card.tsx",
        old_text: "a\nb\nc",
        new_text: "x",
      })]),
      message("new", [tool({
        name: "edit_file",
        summary: "/repo/src/test_ui_card.tsx",
        old_text: "a\nb\nc\nd\ne",
        new_text: "x\ny\nz\n1\n2\n3\n4\n5\n6",
      })]),
    ], { baseDir: "/repo" });

    expect(operations).toHaveLength(1);
    expect(operations[0]).toEqual(expect.objectContaining({
      path: "/repo/src/test_ui_card.tsx",
      additions: 9,
      deletions: 5,
    }));
  });
});

describe("collectMessageFileOperations", () => {
  it("retourne uniquement les fichiers du message donné", () => {
    const oldMessage = message("old", [tool({ summary: "/repo/old.ts", content: "old" })]);
    const currentMessage = message("current", [tool({ summary: "/repo/current.ts", content: "new" })]);

    expect(collectMessageFileOperations(currentMessage).map((operation) => operation.path)).toEqual([
      "/repo/current.ts",
    ]);
    expect(collectMessageFileOperations(oldMessage).map((operation) => operation.path)).toEqual([
      "/repo/old.ts",
    ]);
  });

  it("retourne une liste vide si ce message ne modifie rien", () => {
    const operations = collectMessageFileOperations(message("read", [
      tool({ name: "read_file", summary: "/repo/current.ts", result: "ok" }),
    ]));

    expect(operations).toEqual([]);
  });
});

function message(id: string, tools: ToolActivityRecord[]): AgentMessage {
  return {
    id,
    role: "assistant",
    content: "",
    files: [],
    timestamp: "2026-07-02T10:00:00Z",
    tool_activities: tools,
  };
}

function tool(overrides: Partial<ToolActivityRecord>): ToolActivityRecord {
  return {
    name: "write_file",
    summary: "/repo/file.ts",
    ...overrides,
  };
}
