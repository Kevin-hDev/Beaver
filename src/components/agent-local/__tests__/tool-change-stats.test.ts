import { describe, expect, it } from "vitest";
import type { TFunction } from "i18next";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import { collectFileOperations } from "@/lib/file-preview-utils";
import type {
  AgentMessage,
  ToolActivityRecord,
  ToolFileChangeRecord,
} from "@/types/agent";
import { toolDisplayInfo } from "../tool-display";
import {
  savedToolToRenderable,
  streamToolToRenderable,
} from "../tool-detail-row";

const t = ((key: string) => key) as TFunction;
const PATH = "/repo/src/a.ts";

function change(
  status: ToolFileChangeRecord["status"],
  additions: number,
  deletions: number,
): ToolFileChangeRecord {
  return { path: PATH, status, additions, deletions };
}

function assistant(tool: ToolActivityRecord): AgentMessage {
  return {
    id: "m",
    role: "assistant",
    content: "",
    files: [],
    timestamp: "",
    tool_activities: [tool],
  };
}

function screens(tool: ToolActivityRecord) {
  const bubble = collectFileOperations([assistant(tool)]).reduce(
    (total, operation) => [
      total[0] + operation.additions,
      total[1] + operation.deletions,
    ],
    [0, 0],
  );
  const line = toolDisplayInfo(savedToolToRenderable(tool), "/repo", t);
  return { bulle: bubble, ligneOutil: [line.additions, line.deletions] };
}

function streamLine(tool: ToolActivity) {
  const line = toolDisplayInfo(streamToolToRenderable(tool), "/repo", t);
  return [line.additions, line.deletions];
}

describe("chiffres d'un outil d'écriture", () => {
  it("montre le vrai changement sur la bulle et la ligne pour un edit d'une ligne sur trois", () => {
    expect(
      screens({
        name: "edit_file",
        summary: PATH,
        old_text: "a\nb\nc",
        new_text: "a\nB\nc",
        file_changes: [change("modified", 1, 1)],
      }),
    ).toEqual({ bulle: [1, 1], ligneOutil: [1, 1] });
  });

  it("montre le vrai changement sur la bulle et la ligne quand write_file écrase un fichier", () => {
    expect(
      screens({
        name: "write_file",
        summary: PATH,
        content: "x\ny\n",
        file_changes: [change("modified", 2, 5)],
      }),
    ).toEqual({ bulle: [2, 5], ligneOutil: [2, 5] });
  });

  it("retombe sur le texte de l'outil sans changement enregistré", () => {
    expect(
      screens({
        name: "edit_file",
        summary: PATH,
        old_text: "a\nb\nc",
        new_text: "a\nB\nc",
      }),
    ).toEqual({ bulle: [3, 3], ligneOutil: [3, 3] });
  });

  it("n'affiche aucun chiffre tant que l'outil travaille, puis le vrai changement", () => {
    const running: ToolActivity = {
      name: "edit_file",
      args: { path: PATH, old_string: "a\nb\nc", new_string: "a\nB\nc" },
    };

    expect(streamLine(running)).toEqual([undefined, undefined]);
    expect(
      streamLine({
        ...running,
        result: "ok",
        fileChanges: [change("modified", 1, 1)],
      }),
    ).toEqual([1, 1]);
  });
});
