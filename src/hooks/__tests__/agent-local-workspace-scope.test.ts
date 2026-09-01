import { describe, expect, it } from "vitest";
import type { AgentSessionMeta } from "@/types/agent";
import {
  terminalWorkspaceGroupKey,
  terminalWorkspaceGroupKeys,
} from "../agent-local-workspace-scope";

function session(
  id: string,
  links: Partial<AgentSessionMeta> = {},
): AgentSessionMeta {
  return {
    id,
    name: id,
    created_at: "2026-09-01T00:00:00Z",
    model: "test",
    provider: "test",
    fast_mode_enabled: false,
    message_count: 1,
    ...links,
  };
}

describe("terminalWorkspaceGroupKey", () => {
  it("partage le groupe du projet entre ses discussions", () => {
    const sessions = [
      session("one", { project_id: "project-a" }),
      session("two", { project_id: "project-a" }),
    ];

    expect(terminalWorkspaceGroupKey(sessions[0], sessions, ["project-a"])).toBe("project-a");
    expect(terminalWorkspaceGroupKey(sessions[1], sessions, ["project-a"])).toBe("project-a");
  });

  it("sépare deux discussions sans projet", () => {
    const sessions = [session("one"), session("two")];

    expect(terminalWorkspaceGroupKey(sessions[0], sessions, [])).toBe("session:one");
    expect(terminalWorkspaceGroupKey(sessions[1], sessions, [])).toBe("session:two");
  });

  it("rattache les clones et sous-agents sans projet à leur discussion racine", () => {
    const sessions = [
      session("root"),
      session("clone", { clone_parent_session_id: "root", clone_root_session_id: "root" }),
      session("child", { parent_session_id: "clone" }),
    ];

    expect(terminalWorkspaceGroupKey(sessions[1], sessions, [])).toBe("session:root");
    expect(terminalWorkspaceGroupKey(sessions[2], sessions, [])).toBe("session:root");
  });

  it("traite un projet supprimé comme absent pour le groupe actif et les groupes valides", () => {
    const sessions = [session("orphan", { project_id: "deleted-project" })];

    expect(terminalWorkspaceGroupKey(sessions[0], sessions, [])).toBe("session:orphan");
    expect(terminalWorkspaceGroupKeys(sessions, [])).toEqual(["session:orphan"]);
  });

  it("ne conserve que les groupes des discussions racines et des projets existants", () => {
    const sessions = [
      session("root"),
      session("clone", { clone_parent_session_id: "root", clone_root_session_id: "root" }),
      session("project-chat", { project_id: "project-a" }),
    ];

    expect(terminalWorkspaceGroupKeys(sessions, ["project-a"])).toEqual([
      "project-a",
      "session:root",
    ]);
  });
});
