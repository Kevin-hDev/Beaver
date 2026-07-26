import { describe, expect, it, vi, beforeEach } from "vitest";
import { render } from "@testing-library/react";
import { ConversationList } from "../conversation-list";
import type { AgentSessionMeta, Project } from "@/types/agent";

function makeSession(overrides: Partial<AgentSessionMeta> = {}): AgentSessionMeta {
  return { id: "s1", name: "Test", model: "llama3", provider: "ollama", message_count: 5, created_at: "2026-01-01", ...overrides };
}

function makeProject(overrides: Partial<Project> = {}): Project {
  return { id: "p1", name: "Mon Projet", path: "/tmp/proj", order: 0, created_at: "2026-01-01", ...overrides };
}

const defaultProps = {
  sessions: [] as AgentSessionMeta[], projects: [] as Project[], selectedId: null as string | null,
  onSelect: vi.fn(), onCreate: vi.fn(), onRename: vi.fn(), onDelete: vi.fn(),
  onNewSessionInProject: vi.fn(), onRenameProject: vi.fn(), onDeleteProject: vi.fn(),
  onOpenFolder: vi.fn(), onReorderProjects: vi.fn(),
};

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  Archive: () => <span />,
  Pencil: () => <span />,
  CaretRight: () => <span />,
  DotsThreeVertical: () => <span />,
  ChatsCircle: () => <span />,
  FolderOpen: () => <span />,
  FolderSimple: () => <span />,
  PencilSimple: () => <span />,
}));

vi.mock("@/components/ui/compose-icon", () => ({
  ComposeIcon: () => <span />,
}));

vi.mock("@/components/ui/context-menu", () => ({
  ContextMenu: () => <div />,
}));

vi.mock("../project-section", () => ({
  ProjectSection: (props: { project: Project; collapsed: boolean }) => (
    <div data-testid={`project-${props.project.id}`} data-collapsed={props.collapsed}>
      {props.project.name}
    </div>
  ),
}));

vi.mock("@/hooks/use-keyboard", () => ({
  useKeyboard: () => {},
}));

vi.mock("@/hooks/use-project-drag", () => ({
  useProjectDrag: () => ({
    draggingId: null,
    liveOrder: null,
    onGrab: vi.fn(),
    onHover: vi.fn(),
    onRelease: vi.fn(),
    onCancel: vi.fn(),
  }),
}));

vi.mock("@/hooks/use-session-activity-indicators", () => ({
  useSessionActivityIndicators: () => ({
    runningIds: new Set<string>(),
    unreadIds: new Set<string>(),
    markViewed: vi.fn(),
  }),
}));

vi.mock("../conversation.css", () => ({}));
vi.mock("../conversation-collapse.css", () => ({}));

describe("ConversationList projects", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
  });

  it("affiche les ProjectSection pour chaque projet", () => {
    const p1 = makeProject({ id: "p1", name: "Projet A" });
    const p2 = makeProject({ id: "p2", name: "Projet B" });
    const { getByTestId } = render(<ConversationList {...defaultProps} projects={[p1, p2]} />);

    expect(getByTestId("project-p1")).not.toBeNull();
    expect(getByTestId("project-p2")).not.toBeNull();
  });

  it("restaure un projet replié depuis le stockage local", () => {
    localStorage.setItem("clgo-conversation-collapse-v1", JSON.stringify({
      projectsCollapsed: false,
      discussionsCollapsed: false,
      collapsedProjectIds: ["p1"],
    }));
    const { getByTestId } = render(<ConversationList {...defaultProps} projects={[makeProject()]} />);

    expect(getByTestId("project-p1").getAttribute("data-collapsed")).toBe("true");
  });

  it("affiche comme orpheline une session dont le project_id ne correspond à aucun projet", () => {
    const session = makeSession({ id: "s1", project_id: "projet-inconnu" });
    const { container } = render(<ConversationList {...defaultProps} sessions={[session]} projects={[]} />);

    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
  });

  it("n'affiche pas dans les orphelins une session assignée à un projet existant", () => {
    const projet = makeProject({ id: "p1" });
    const session = makeSession({ id: "s1", project_id: "p1" });
    const { container } = render(<ConversationList {...defaultProps} sessions={[session]} projects={[projet]} />);

    expect(container.querySelectorAll(".conv-session-indented").length).toBe(0);
  });
});
