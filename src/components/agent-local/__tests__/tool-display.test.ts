import { describe, expect, it } from "vitest";
import { filePathSegments, shortenPath, toolDisplayInfo } from "../tool-display";
import { savedToolToRenderable, streamToolToRenderable } from "../tool-detail-row";
import type { RenderableTool } from "../tool-detail-row";
import type { TFunction } from "i18next";

const t = ((key: string) => ({
  "agentLocal.toolActivity.actions.read": "Read",
  "agentLocal.toolActivity.actions.create": "Create",
  "agentLocal.toolActivity.actions.edit": "Edit",
  "agentLocal.toolActivity.actions.skill": "Skill",
  "agentLocal.toolActivity.actions.agent": "Agent",
  "agentLocal.toolActivity.actions.forecast": "Forecast",
  "agentLocal.toolActivity.actions.mcp": "MCP",
  "agentLocal.toolActivity.actions.list": "List",
  "agentLocal.toolActivity.actions.search": "Search",
  "agentLocal.toolActivity.actions.run": "Run",
  "agentLocal.toolActivity.actions.stopProcess": "Process stopped",
}[key] ?? key)) as TFunction;

describe("toolDisplayInfo", () => {
  it("raccourcit les chemins depuis la racine projet", () => {
    const path = "/Users/kevinh/Projects/systeme-pulse/src/App.tsx";
    expect(shortenPath(path, "/Users/kevinh/Projects/systeme-pulse")).toBe("systeme-pulse/src/App.tsx");
  });

  it("limite à 3 dossiers parents maximum", () => {
    const deep = "/Users/kevinh/Projects/systeme-pulse/a/b/c/d/e/file.ts";
    expect(shortenPath(deep, "/Users/kevinh/Projects/systeme-pulse")).toBe("…/c/d/e/file.ts");
  });

  it("garde le chemin complet s'il a 3 dossiers ou moins", () => {
    const shallow = "/Users/kevinh/Projects/systeme-pulse/src/main.rs";
    expect(shortenPath(shallow, "/Users/kevinh/Projects/systeme-pulse")).toBe("systeme-pulse/src/main.rs");
  });
});

describe("filePathSegments", () => {
  it("sépare les dossiers (tronçables) du nom de fichier (fixe)", () => {
    const seg = filePathSegments(
      "/Users/kevinh/Projects/systeme-pulse/a/b/c/d/e/file.ts",
      "/Users/kevinh/Projects/systeme-pulse",
    );
    expect(seg.dirs).toBe("…/c/d/e/");
    expect(seg.fileName).toBe("file.ts");
  });

  it("retourne dirs vide si seulement le nom de fichier", () => {
    const seg = filePathSegments("config.json", undefined);
    expect(seg.dirs).toBe("");
    expect(seg.fileName).toBe("config.json");
  });
});

describe("toolDisplayInfo", () => {
  it("affiche une création avec stats + lignes + icône + segments chemin", () => {
    const tool: RenderableTool = {
      name: "write_file",
      summary: "/Users/kevinh/Projects/systeme-pulse/vite.config.ts",
      content: "a\nb\n",
    };
    expect(toolDisplayInfo(tool, "/Users/kevinh/Projects/systeme-pulse", t)).toEqual({
      label: "Create",
      summary: "systeme-pulse/vite.config.ts",
      additions: 2,
      deletions: 0,
      icon: "FilePlus",
      dir: "systeme-pulse/",
      fileName: "vite.config.ts",
    });
  });

  it("affiche une modification avec stats old/new + icône + segments chemin", () => {
    const tool: RenderableTool = {
      name: "edit_file",
      summary: "/Users/kevinh/Projects/systeme-pulse/src/main.rs",
      old_text: "a\nb\nc",
      new_text: "a",
    };
    expect(toolDisplayInfo(tool, "/Users/kevinh/Projects/systeme-pulse", t)).toEqual({
      label: "Edit",
      summary: "systeme-pulse/src/main.rs",
      additions: 1,
      deletions: 3,
      icon: "Pencil",
      dir: "systeme-pulse/src/",
      fileName: "main.rs",
    });
  });

  it("ne change pas web_search", () => {
    expect(toolDisplayInfo({ name: "web_search", summary: "tauri docs" }, undefined, t)).toEqual({
      label: "web_search",
      summary: "tauri docs",
      icon: "Globe",
    });
  });

  it("garde bash comme nom de tool affiché", () => {
    expect(toolDisplayInfo({ name: "bash", summary: "npm test" }, undefined, t)).toEqual({
      label: "bash",
      summary: "npm test",
      icon: "TerminalWindow",
    });
    expect(toolDisplayInfo({ name: "bash_write", summary: "session-1" }, undefined, t)).toEqual({
      label: "bash",
      summary: "session-1",
      icon: "TerminalWindow",
    });
  });

  it("affiche un arrêt volontaire comme une action dédiée", () => {
    expect(toolDisplayInfo({
      name: "bash_write",
      summary: "npm start",
      args: { session_id: "session-1", stop: true },
    }, undefined, t)).toEqual({
      label: "Process stopped",
      summary: "",
      icon: "TerminalWindow",
    });
  });

  it("reclasse l'ancien résultat d'arrêt sans masquer les autres erreurs", () => {
    const sessionId = "6a719eeb-1665-49cd-a5e2-23427e80543b";
    const tool = savedToolToRenderable({
      name: "bash_write",
      summary: sessionId,
      args: { session_id: sessionId, stop: true },
      result: "Commande annulee.",
      is_error: true,
    });

    expect(tool.is_error).toBe(false);
    expect(tool.legacySuccessfulStop).toBe(true);
  });

  it("affiche les tools spécialisés avec des noms explicites", () => {
    expect(toolDisplayInfo({ name: "load_skill", summary: "context7-docs" }, undefined, t).label).toBe("Skill");
    expect(toolDisplayInfo({ name: "delegate_task", summary: "audit" }, undefined, t).label).toBe("Agent");
    expect(toolDisplayInfo({ name: "forecast_data_audit", summary: "sales" }, undefined, t).label).toBe("Forecast");
    expect(toolDisplayInfo({ name: "forecast", summary: "sales" }, undefined, t).label).toBe("Read");
    expect(toolDisplayInfo({ name: "forecast_models", summary: "models" }, undefined, t).label).toBe("Read");
    expect(toolDisplayInfo({ name: "forecast_read", summary: "analysis" }, undefined, t).label).toBe("Read");
    expect(toolDisplayInfo({ name: "forecast_analyze", summary: "scenario" }, undefined, t).label).toBe("Forecast");
    expect(toolDisplayInfo({ name: "search_mcp_tools", summary: "linear" }, undefined, t).label).toBe("MCP");
  });

  it("affiche uniquement le nom lisible du skill pendant le stream", () => {
    const tool = streamToolToRenderable({
      name: "load_skill",
      args: { skill_id: "local:skill:0123456789abcdef" },
      displaySummary: "context7-docs",
    });

    expect(tool.summary).toBe("context7-docs");
  });

  it("masque les identifiants opaques des anciens historiques", () => {
    const tool = savedToolToRenderable({
      name: "load_skill",
      summary: '{"skill_id":"claude:skill:0123456789abcdef"}',
    });

    expect(tool.summary).toBe("");
  });

  it("tronque les commandes bash longues sur une seule ligne", () => {
    const command = `${"a".repeat(110)}\necho done`;
    expect(toolDisplayInfo({ name: "bash", summary: command }, undefined, t)).toEqual({
      label: "bash",
      summary: `${"a".repeat(96)}...`,
      icon: "TerminalWindow",
    });
  });

  it("associe la bonne icône Phosphor à chaque type d'outil", () => {
    const cases: Array<[string, string]> = [
      ["read_file", "BookOpenText"],
      ["read_spreadsheet", "FileText"],
      ["read_document", "FileText"],
      ["read_image", "Image"],
      ["write_file", "FilePlus"],
      ["write_spreadsheet", "FilePlus"],
      ["write_document", "FilePlus"],
      ["edit_file", "Pencil"],
      ["process_image", "Pencil"],
      ["bash", "TerminalWindow"],
      ["bash_write", "TerminalWindow"],
      ["web_search", "Globe"],
      ["web_fetch", "Link"],
      ["list_dir", "FolderOpen"],
      ["grep", "MagnifyingGlass"],
      ["glob", "MagnifyingGlass"],
      ["create_branch", "GitBranch"],
      ["checkout_branch", "GitBranch"],
      ["load_skill", "Sparkle"],
      ["delegate_task", "Users"],
      ["forecast_data_audit", "ChartLineUp"],
      ["forecast", "ChartLineUp"],
      ["forecast_analyze", "ChartLineUp"],
      ["search_mcp_tools", "Plugs"],
    ];
    for (const [name, expectedIcon] of cases) {
      const info = toolDisplayInfo({ name, summary: "x" }, undefined, t);
      expect(info.icon).toBe(expectedIcon);
    }
  });

  it("retourne l'icône Wrench par défaut pour un outil inconnu", () => {
    expect(toolDisplayInfo({ name: "outil_inconnu", summary: "x" }, undefined, t).icon).toBe("Wrench");
  });
});
