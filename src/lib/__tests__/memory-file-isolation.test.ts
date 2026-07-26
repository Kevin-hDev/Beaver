import { describe, expect, it } from "vitest";
import { toolToFileOperations } from "../file-preview-operation-builder";
import { summarizeToolChange } from "../session-summary";
import type { ToolActivityRecord } from "@/types/agent";

const memoryWrite: ToolActivityRecord = {
  name: "write_file",
  summary: "/memory/global/topics/preference.md",
  resolved_path: "/memory/global/topics/preference.md",
  domain: "memory",
  content: "# Préférence",
  result: "ok",
  is_error: false,
};

describe("isolation des fichiers MEMORY", () => {
  it("ne les ajoute pas aux opérations de fichiers du projet", () => {
    expect(toolToFileOperations(
      memoryWrite,
      "message-1",
      0,
      "2026-07-24T20:00:00Z",
    )).toEqual([]);
  });

  it("ne les compte pas dans le résumé des changements du projet", () => {
    expect(summarizeToolChange(memoryWrite)).toEqual({
      additions: 0,
      deletions: 0,
      files: 0,
    });
  });
});
