import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { ProjectSection } from "../project-section";
import type { Project } from "@/types/agent";

const project: Project = {
  id: "p1",
  name: "Projet",
  path: "/tmp/projet",
  order: 0,
  created_at: "2026-01-01",
};

const baseProps = {
  project,
  sessions: [],
  selectedId: null,
  runningIds: new Set<string>(),
  unreadIds: new Set<string>(),
  isDragOver: false,
  onSelect: vi.fn(),
  onNewSession: vi.fn(),
  onRenameProject: vi.fn(),
  onDeleteProject: vi.fn(),
  onOpenFolder: vi.fn(),
  onRenameSession: vi.fn(),
  onDeleteSession: vi.fn(),
  onGrab: vi.fn(),
  isDragging: false,
  onToggleCollapse: vi.fn(),
  nowMs: Date.UTC(2026, 5, 30, 12, 0, 0),
};

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  Archive: () => <span />,
  DotsThreeVertical: () => <span />,
  FolderOpen: () => <span />,
  PencilSimple: () => <span />,
  X: () => <span />,
}));

vi.mock("@/components/ui/folder-state-icon", () => ({
  FolderStateIcon: ({ open }: { open: boolean }) => (
    <span data-testid="folder-state-icon" data-open={String(open)} />
  ),
}));

vi.mock("@/components/ui/compose-icon", () => ({
  ComposeIcon: () => <span />,
}));

vi.mock("@/components/ui/context-menu", () => ({
  ContextMenu: () => <div />,
}));

vi.mock("../conversation-session-item", () => ({
  ConversationSessionItem: () => <div />,
}));

vi.mock("@/hooks/use-keyboard", () => ({
  useKeyboard: () => {},
}));

afterEach(() => {
  cleanup();
});

describe("ProjectSection", () => {
  it("affiche l'icône dossier ouvert quand le projet est déplié", () => {
    const { getByTestId } = render(<ProjectSection {...baseProps} collapsed={false} />);

    expect(getByTestId("folder-state-icon").getAttribute("data-open")).toBe("true");
  });

  it("affiche l'icône dossier fermé quand le projet est replié", () => {
    const { getByTestId } = render(<ProjectSection {...baseProps} collapsed />);

    expect(getByTestId("folder-state-icon").getAttribute("data-open")).toBe("false");
  });
});
