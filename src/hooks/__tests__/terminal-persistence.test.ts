import { beforeEach, describe, expect, it, vi } from "vitest";
import { loadSavedGroups, saveGroups } from "../terminal-persistence";
import type { TerminalGroup } from "../terminal-types";

const calls: { op: string; path: string; content?: string }[] = [];

vi.mock("@tauri-apps/api/path", () => ({
  homeDir: () => Promise.resolve("/home/test"),
  join: (...parts: string[]) => Promise.resolve(parts.join("/")),
}));

let renameFailures = 0;

vi.mock("@tauri-apps/plugin-fs", () => ({
  readTextFile: (path: string) => {
    calls.push({ op: "read", path });
    if (path.includes("corrupt")) return Promise.reject(new Error("not found"));
    return Promise.resolve("{}");
  },
  writeTextFile: (path: string, content: string) => {
    calls.push({ op: "write", path, content });
    return Promise.resolve();
  },
  rename: (from: string, to: string) => {
    calls.push({ op: "rename", path: `${from} -> ${to}` });
    if (renameFailures > 0) {
      renameFailures -= 1;
      return Promise.reject(new Error("dest exists"));
    }
    return Promise.resolve();
  },
  remove: (path: string) => {
    calls.push({ op: "remove", path });
    return Promise.resolve();
  },
}));

function groupWith(label: string): TerminalGroup {
  return {
    tabs: [{ id: "t1", ptyId: 1, ptyToken: "tok", label, cwd: "/tmp", hasActivity: false }],
    activeTabId: "t1",
  };
}

describe("saveGroups", () => {
  beforeEach(() => {
    calls.length = 0;
    renameFailures = 0;
  });

  it("écrit dans un fichier temporaire puis renomme (écriture atomique)", async () => {
    const groups = new Map([["proj", groupWith("build")]]);

    await saveGroups(groups);

    const ops = calls.map((c) => c.op);
    expect(ops).toEqual(["write", "rename"]);
    expect(calls[0].path).toMatch(/terminal-tabs\.json\.tmp$/);
    expect(calls[0].content).toBe(JSON.stringify({ proj: [{ label: "build", cwd: "/tmp" }] }));
    expect(calls[1].path).toMatch(/terminal-tabs\.json\.tmp -> .*terminal-tabs\.json$/);
  });

  it("ne persiste jamais les identifiants ni jetons PTY", async () => {
    await saveGroups(new Map([["proj", groupWith("build")]]));

    expect(calls[0].content).not.toContain("ptyId");
    expect(calls[0].content).not.toContain("tok");
  });

  it("retire la destination avant de renommer quand Windows refuse l'écrasement", async () => {
    renameFailures = 1;

    await saveGroups(new Map([["proj", groupWith("build")]]));

    expect(calls.map((c) => c.op)).toEqual(["write", "rename", "remove", "rename"]);
  });

  it("avale une erreur d'écriture sans propager", async () => {
    renameFailures = 99;

    await expect(saveGroups(new Map([["proj", groupWith("build")]]))).resolves.toBeUndefined();
  });
});

describe("loadSavedGroups", () => {
  beforeEach(() => {
    calls.length = 0;
  });

  it("renvoie un objet vide quand le fichier est illisible", async () => {
    expect(await loadSavedGroups()).toEqual({});
  });
});
