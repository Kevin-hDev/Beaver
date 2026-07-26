import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ArchivedChatsSettings } from "../archived-chats-settings";
import type { AgentSessionMeta, Project } from "@/types/agent";

const restore = vi.fn();
const remove = vi.fn();

const project: Project = {
  id: "p1",
  name: "CL-GO-DASH",
  path: "/tmp/project",
  order: 0,
  created_at: "2026-01-01T00:00:00Z",
};

function session(id: string, projectId?: string): AgentSessionMeta {
  return {
    id,
    name: `Session ${id}`,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: `2026-01-${id.padStart(2, "0")}T00:00:00Z`,
    archived_at: "2026-02-01T00:00:00Z",
    model: "llama3",
    provider: "ollama",
    message_count: 1,
    project_id: projectId,
  };
}

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "fr" },
    t: (key: string, opts?: { count?: number }) => key === "settings.archivedChats.count"
      ? `${opts?.count ?? 0} discussion(s)`
      : key,
  }),
}));

vi.mock("@/hooks/use-archived-agent-sessions", () => ({
  useArchivedAgentSessions: () => ({
    sessions: [session("1", "p1"), ...Array.from({ length: 7 }, (_, index) => session(String(index + 2)))],
    loading: false,
    restore,
    remove,
  }),
}));

vi.mock("@/hooks/use-projects", () => ({
  useProjects: () => ({ projects: [project] }),
}));

vi.mock("@/components/ui/icons", () => ({
  Archive: () => <span />,
  ChatsCircle: () => <span />,
  FolderSimple: () => <span />,
  Search: () => <span />,
  Trash: () => <span />,
  CaretDown: () => <span />,
  MagnifyingGlass: () => <span />,
}));

vi.mock("../settings-select", () => ({
  SettingsSelect: ({ value, onChange }: { value: string; onChange: (value: string) => void }) => (
    <button data-testid="filter" onClick={() => onChange("__discussions__")}>{value}</button>
  ),
}));

vi.mock("../archived-chats-settings.css", () => ({}));
vi.mock("../archived-chats-settings-controls.css", () => ({}));

describe("ArchivedChatsSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("groupe les archives par projet et discussions", () => {
    const { container } = render(<ArchivedChatsSettings />);

    expect(screen.getByText("CL-GO-DASH")).not.toBeNull();
    expect(screen.getByText("projects.discussions")).not.toBeNull();
    expect(container.querySelectorAll(".acs-bubble").length).toBe(2);
    expect(container.querySelector(".acs-session-list.is-scrollable")).not.toBeNull();
  });

  it("filtre sur les discussions simples", () => {
    const { container } = render(<ArchivedChatsSettings />);

    fireEvent.click(screen.getByTestId("filter"));

    expect(screen.queryByText("CL-GO-DASH")).toBeNull();
    expect(screen.getByText("projects.discussions")).not.toBeNull();
    expect(container.querySelectorAll(".acs-bubble").length).toBe(1);
  });

  it("demande une confirmation avant de supprimer une archive", async () => {
    render(<ArchivedChatsSettings />);

    fireEvent.click(screen.getAllByLabelText("settings.archivedChats.delete")[0]);

    expect(remove).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText("settings.archivedChats.confirmDeleteButton"));

    await waitFor(() => expect(remove).toHaveBeenCalledWith("1"));
  });

  it("demande une confirmation avant de tout supprimer", async () => {
    render(<ArchivedChatsSettings />);

    fireEvent.click(screen.getByLabelText("settings.archivedChats.deleteAll"));

    expect(remove).not.toHaveBeenCalled();
    fireEvent.click(screen.getByText("settings.archivedChats.confirmDeleteButton"));

    await waitFor(() => expect(remove).toHaveBeenCalledTimes(8));
  });
});
