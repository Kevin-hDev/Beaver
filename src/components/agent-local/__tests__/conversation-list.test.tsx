import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent } from "@testing-library/react";
import { ConversationList } from "../conversation-list";
import type { AgentSessionMeta, Project } from "@/types/agent";

const activityMocks = vi.hoisted(() => ({
  runningIds: new Set<string>(),
  unreadIds: new Set<string>(),
  markViewed: vi.fn(),
}));

function makeSession(overrides: Partial<AgentSessionMeta> = {}): AgentSessionMeta {
  return { id: "s1", name: "Test", model: "llama3", provider: "ollama", message_count: 5, created_at: "2026-01-01", ...overrides };
}
const defaultProps = {
  sessions: [] as AgentSessionMeta[], projects: [] as Project[], selectedId: null as string | null,
  onSelect: vi.fn(), onCreate: vi.fn(), onRename: vi.fn(), onDelete: vi.fn(),
  onNewSessionInProject: vi.fn(), onRenameProject: vi.fn(), onDeleteProject: vi.fn(),
  onOpenFolder: vi.fn(), onReorderProjects: vi.fn(), onReorderSessions: vi.fn(),
  onTogglePin: vi.fn(), onReorderPinnedSessions: vi.fn(),
};

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  Archive: () => <span />,
  Pencil: () => <span />,
  CaretRight: () => <span data-testid="section-chevron" />,
  DotsThreeVertical: () => <span data-testid="dots" />,
  FolderOpen: () => <span />,
  FolderSimple: () => <span />,
  PencilSimple: () => <span />,
}));

vi.mock("@/components/ui/compose-icon", () => ({
  ComposeIcon: () => <span data-testid="compose" />,
}));

vi.mock("@/components/ui/context-menu", () => ({
  ContextMenu: () => <div data-testid="context-menu" />,
}));

vi.mock("../project-section", () => ({
  ProjectSection: (props: { project: Project }) => (
    <div data-testid={`project-${props.project.id}`}>{props.project.name}</div>
  ),
}));

vi.mock("@/hooks/use-keyboard", () => ({
  useKeyboard: () => {},
}));

vi.mock("@/hooks/use-session-activity-indicators", () => ({
  useSessionActivityIndicators: () => activityMocks,
}));

vi.mock("../conversation.css", () => ({}));
vi.mock("../conversation-collapse.css", () => ({}));

describe("ConversationList", () => {
	  beforeEach(() => {
	    vi.clearAllMocks();
	    activityMocks.runningIds = new Set<string>();
	    activityMocks.unreadIds = new Set<string>();
	  });

  it("ouvre le champ de renommage au double-clic sur le nom d'une conversation", () => {
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[makeSession()]} />,
    );

    fireEvent.doubleClick(container.querySelector(".conv-session-main") as HTMLElement);

    expect(container.querySelector("input.conv-rename")).not.toBeNull();
  });

  it("affiche le bouton nouveau avec la classe .conv-new-btn", () => {
    const { container } = render(<ConversationList {...defaultProps} />);
    expect(container.querySelector(".conv-new-btn")).not.toBeNull();
  });

  it("affiche le message vide si aucune session et aucun projet", () => {
    const { container } = render(<ConversationList {...defaultProps} />);
    const empty = container.querySelector(".hist-empty");
    expect(empty).not.toBeNull();
    expect(empty?.textContent).toContain("agentLocal.noConversations");
  });

  it("affiche les sessions orphelines sans project_id", () => {
    const session = makeSession({ id: "s1", name: "Session orpheline" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} />,
    );
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
  });

  it("n'affiche pas les sous-sessions avec parent_session_id", () => {
    const parent = makeSession({ id: "parent" });
    const child = makeSession({ id: "child", parent_session_id: "parent" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[parent, child]} />,
    );
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
  });

  it("n'affiche pas les clones avec clone_parent_session_id", () => {
    const parent = makeSession({ id: "parent" });
    const clone = makeSession({ id: "clone", clone_parent_session_id: "parent" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[parent, clone]} />,
    );
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
  });

  it("marque la session active avec la classe .active", () => {
    const session = makeSession({ id: "s1" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} selectedId="s1" />,
    );
    expect(container.querySelector(".conv-session-indented.active")).not.toBeNull();
  });

  it("ne marque pas les autres sessions comme .active", () => {
    const s1 = makeSession({ id: "s1" });
    const s2 = makeSession({ id: "s2", name: "Autre" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[s1, s2]} selectedId="s1" />,
    );
    const allItems = container.querySelectorAll(".conv-session-indented");
    const inactives = Array.from(allItems).filter((el) => !el.classList.contains("active"));
    expect(inactives.length).toBe(1);
  });

	  it("appelle onSelect au clic sur une session", () => {
    const onSelect = vi.fn();
    const session = makeSession({ id: "s42" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} onSelect={onSelect} />,
    );
	    fireEvent.click(container.querySelector(".conv-session-indented") as HTMLElement);
	    expect(onSelect).toHaveBeenCalledWith("s42");
	  });

	  it("retire l'indicateur terminé au clic sur une session", () => {
	    const onSelect = vi.fn();
	    const session = makeSession({ id: "s42" });
	    const { container } = render(
	      <ConversationList {...defaultProps} sessions={[session]} onSelect={onSelect} />,
	    );
	    fireEvent.click(container.querySelector(".conv-session-indented") as HTMLElement);
	    expect(activityMocks.markViewed).toHaveBeenCalledWith("s42");
	    expect(onSelect).toHaveBeenCalledWith("s42");
	  });

  it("appelle onCreate au clic sur le bouton nouveau", () => {
    const onCreate = vi.fn();
    const { container } = render(
      <ConversationList {...defaultProps} onCreate={onCreate} />,
    );
    fireEvent.click(container.querySelector(".conv-new-btn") as HTMLElement);
    expect(onCreate).toHaveBeenCalledOnce();
  });

  it("affiche la clé i18n pour une session nommée 'Nouvelle session'", () => {
    const session = makeSession({ id: "s1", name: "Nouvelle session" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} />,
    );
    expect(container.querySelector(".conv-name")?.textContent).toBe("agentLocal.newSession");
  });

	  it("affiche l'âge de la session dans la zone droite", () => {
    const createdAt = new Date(Date.now() - 5 * 60_000).toISOString();
    const session = makeSession({ id: "s1", created_at: createdAt });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} />,
    );
    expect(container.querySelector(".conv-session-age")?.textContent).toBe("sessionAge.minute");
	    expect(container.querySelector(".conv-session-menu-btn")).not.toBeNull();
	  });

	  it("ouvre le menu de session au clic sur les trois points", () => {
    const session = makeSession({ id: "s1" });
    const { container, getByTestId } = render(
      <ConversationList {...defaultProps} sessions={[session]} />,
    );
    fireEvent.click(container.querySelector(".conv-session-menu-btn") as HTMLElement);
    expect(getByTestId("context-menu")).not.toBeNull();
  });

  /* L'icône de session marquait l'état actif par son remplissage et a été
     retirée de la ligne. La distinction tient maintenant à la seule classe
     active, que le CSS traduit en fond et en couleur d'encre. */
  it("distingue la session active des autres", () => {
    const s1 = makeSession({ id: "s1" });
    const s2 = makeSession({ id: "s2", name: "Autre" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[s1, s2]} selectedId="s1" />,
    );

    const actives = container.querySelectorAll(".conv-session-indented.active");

    expect(container.querySelectorAll(".conv-session-indented")).toHaveLength(2);
    expect(actives).toHaveLength(1);
    expect(actives[0].getAttribute("data-nav-active")).toBe("true");
  });

  it("n'affiche pas de section Épinglé quand rien n'est épinglé", () => {
    const { container, queryByText } = render(
      <ConversationList {...defaultProps} sessions={[makeSession()]} />,
    );
    expect(queryByText("projects.pinned")).toBeNull();
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
  });

  it("affiche les épinglées dans une section en tête, et plus dans Discussions", () => {
    const pinned = makeSession({ id: "pin", name: "Épinglée", pinned_at: "2026-08-21T10:00:00Z" });
    const other = makeSession({ id: "s2", name: "Autre" });
    const { container, getByText } = render(
      <ConversationList {...defaultProps} sessions={[pinned, other]} />,
    );

    const labels = Array.from(container.querySelectorAll(".conv-section-label")).map((el) => el.textContent);
    expect(labels[0]).toBe("projects.pinned");
    expect(getByText("projects.discussions")).not.toBeNull();
    const pinnedSection = getByText("projects.pinned").parentElement as HTMLElement;
    expect(pinnedSection.querySelectorAll(".conv-session-indented").length).toBe(1);
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(2);
  });

  it("garde une session de projet épinglée hors de son projet", () => {
    const project: Project = { id: "p1", name: "Projet", path: "/tmp/p", order: 0, created_at: "2026-01-01" };
    const pinned = makeSession({ id: "pin", project_id: "p1", pinned_at: "2026-08-21T10:00:00Z" });
    const { container, getByTestId } = render(
      <ConversationList {...defaultProps} sessions={[pinned]} projects={[project]} />,
    );

    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
    expect(getByTestId("project-p1")).not.toBeNull();
  });

  it("range la section Épinglé avant les projets", () => {
    const project: Project = { id: "p1", name: "Projet", path: "/tmp/p", order: 0, created_at: "2026-01-01" };
    const pinned = makeSession({ id: "pin", pinned_at: "2026-08-21T10:00:00Z" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[pinned]} projects={[project]} />,
    );

    const labels = Array.from(container.querySelectorAll(".conv-section-label")).map((el) => el.textContent);
    expect(labels).toEqual(["projects.pinned", "projects.title"]);
  });

  it("n'affiche pas les sous-agents (parent_session_id défini) même sans project_id", () => {
    // Un sous-agent est filtré de la liste visible (mainSessions exclut parent_session_id)
    // Mais sessions.length > 0 donc le message "aucune conversation" n'apparaît pas
    const parent = makeSession({ id: "parent" });
    const subAgent = makeSession({ id: "sub", parent_session_id: "parent" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[parent, subAgent]} />,
    );
    // Seul le parent apparaît dans la liste orpheline, pas le sous-agent
    const items = container.querySelectorAll(".conv-session-indented");
    expect(items.length).toBe(1);
    expect(items[0].getAttribute("data-session-id") ?? items[0].textContent).not.toContain("sub");
  });
});
