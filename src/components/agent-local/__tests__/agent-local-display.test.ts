import { describe, expect, it } from "vitest";
import type { AgentSessionMeta, Project } from "@/types/agent";
import {
  resolveDisplayModel, resolveDisplayProject, resolveDisplayReasoningMode, resolveDisplaySession,
} from "../agent-local-display";

describe("agent local display resolution", () => {
  const sessions: AgentSessionMeta[] = [
    { id: "main", name: "Main", model: "m", provider: "p", message_count: 0, created_at: "2026-01-01" },
    { id: "clone", name: "Clone", model: "m", provider: "p", message_count: 0, created_at: "2026-01-01", project_id: "p2" },
  ];
  const projects: Project[] = [
    { id: "p1", name: "One", path: "/one", order: 0, created_at: "2026-01-01" },
    { id: "p2", name: "Two", path: "/two", order: 1, created_at: "2026-01-01" },
  ];

  it("uses the selected tab session and its project", () => {
    const session = resolveDisplaySession(sessions, "clone", sessions[0]);
    const project = resolveDisplayProject(projects, session, projects[0]);

    expect(session?.id).toBe("clone");
    expect(project?.id).toBe("p2");
  });

  it("falls back safely when the selected tab is unavailable", () => {
    const session = resolveDisplaySession(sessions, "missing", sessions[0]);
    const project = resolveDisplayProject(projects, session, projects[0]);

    expect(session?.id).toBe("main");
    expect(project?.id).toBe("p1");
    expect(resolveDisplaySession(sessions, null, sessions[0])).toBeNull();
  });

  it("keeps the session reasoning choice ahead of the default", () => {
    expect(resolveDisplayReasoningMode({ ...sessions[0], reasoning_mode: "high" }, "low")).toBe("high");
    expect(resolveDisplayReasoningMode({ ...sessions[0], thinking_enabled: true }, "low")).toBe("auto");
    expect(resolveDisplayReasoningMode(null, "low")).toBe("low");
  });

  it("keeps the selected session model ahead of the default", () => {
    expect(resolveDisplayModel(sessions[0], "default-model", "default-provider")).toEqual({
      displayModel: "m",
      displayProvider: "p",
    });
  });
});
