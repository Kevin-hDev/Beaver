import { describe, it, expect } from "vitest";
import { isHiddenAgentTool } from "./hidden-agent-tools";

describe("isHiddenAgentTool", () => {
  it("retourne true pour les outils todo cachés", () => {
    expect(isHiddenAgentTool("todo_write")).toBe(true);
    expect(isHiddenAgentTool("todo_history")).toBe(true);
    expect(isHiddenAgentTool("todo_pause")).toBe(true);
    expect(isHiddenAgentTool("todo_resume")).toBe(true);
    expect(isHiddenAgentTool("todo_delete")).toBe(true);
  });

  it("retourne true pour agent_diagnostics", () => {
    expect(isHiddenAgentTool("agent_diagnostics")).toBe(true);
  });

  it("retourne true pour ask_user_choice", () => {
    expect(isHiddenAgentTool("ask_user_choice")).toBe(true);
  });

  it("retourne true pour planmode et exitplanmode", () => {
    expect(isHiddenAgentTool("planmode")).toBe(true);
    expect(isHiddenAgentTool("exitplanmode")).toBe(true);
  });

  it("retourne false pour les outils visibles (bash, read_file)", () => {
    expect(isHiddenAgentTool("bash")).toBe(false);
    expect(isHiddenAgentTool("read_file")).toBe(false);
    expect(isHiddenAgentTool("write_file")).toBe(false);
    expect(isHiddenAgentTool("web_search")).toBe(false);
  });

  it("retourne false pour un nom d'outil inconnu", () => {
    expect(isHiddenAgentTool("unknown_tool")).toBe(false);
  });

  it("retourne false pour une chaîne vide", () => {
    expect(isHiddenAgentTool("")).toBe(false);
  });

  it("est sensible à la casse", () => {
    // todo_write est caché, mais TODO_WRITE ne l'est pas.
    expect(isHiddenAgentTool("TODO_WRITE")).toBe(false);
    expect(isHiddenAgentTool("Todo_Write")).toBe(false);
  });
});
