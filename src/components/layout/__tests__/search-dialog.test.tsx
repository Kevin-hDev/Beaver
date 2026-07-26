import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SearchDialog } from "../search-dialog";
import type { AgentSessionMeta, Project } from "@/types/agent";

const mocks = vi.hoisted(() => ({
  sessions: [] as AgentSessionMeta[],
  projects: [] as Project[],
}));

function session(id: string, name = `Session ${id}`, extra: Partial<AgentSessionMeta> = {}): AgentSessionMeta {
  return {
    id,
    name,
    created_at: `2026-01-${id.padStart(2, "0")}`,
    model: "llama3",
    provider: "ollama",
    message_count: 1,
    ...extra,
  };
}

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/hooks/use-agent-sessions", () => ({
  useAgentSessions: () => ({ sessions: mocks.sessions }),
}));

vi.mock("@/hooks/use-projects", () => ({
  useProjects: () => ({ projects: mocks.projects }),
}));

describe("SearchDialog", () => {
  beforeEach(() => {
    mocks.sessions = [];
    mocks.projects = [];
  });

  afterEach(() => {
    cleanup();
  });

  it("exclut les sous-sessions internes", () => {
    mocks.sessions = [
      session("1", "Visible"),
      session("2", "Subagent hidden", { parent_session_id: "1" }),
    ];

    render(<SearchDialog open onClose={vi.fn()} onSelect={vi.fn()} />);

    expect(screen.getByText("Visible")).not.toBeNull();
    expect(screen.queryByText("Subagent hidden")).toBeNull();
  });

  it("borne les résultats de recherche affichés", () => {
    mocks.sessions = Array.from({ length: 60 }, (_, i) => session(String(i + 1), `Match ${i + 1}`));

    render(<SearchDialog open onClose={vi.fn()} onSelect={vi.fn()} />);
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "match" } });

    expect(screen.getAllByRole("option")).toHaveLength(50);
  });

  it("borne la longueur de la requête", () => {
    render(<SearchDialog open onClose={vi.fn()} onSelect={vi.fn()} />);
    const input = screen.getByRole("textbox");

    fireEvent.change(input, { target: { value: "a".repeat(150) } });

    expect(input).toHaveProperty("value", "a".repeat(120));
  });

  it("sécurise Entrée quand la liste filtrée devient plus courte", () => {
    const onSelect = vi.fn();
    mocks.sessions = [
      session("1", "Alpha"),
      session("2", "Beta"),
      session("3", "Gamma"),
    ];

    render(<SearchDialog open onClose={vi.fn()} onSelect={onSelect} />);
    fireEvent.keyDown(window, { key: "ArrowDown" });
    fireEvent.keyDown(window, { key: "ArrowDown" });
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "alpha" } });
    fireEvent.keyDown(window, { key: "Enter" });

    expect(onSelect).toHaveBeenCalledWith("1");
  });

  it("expose un vrai rôle de dialogue modal", () => {
    render(<SearchDialog open onClose={vi.fn()} onSelect={vi.fn()} />);

    expect(screen.getByRole("dialog").getAttribute("aria-modal")).toBe("true");
  });
});
