import { describe, expect, it, vi } from "vitest";
import { act, fireEvent, render } from "@testing-library/react";
import { SessionSummaryBubble } from "../session-summary-bubble";
import type { SessionSummaryHookState } from "@/hooks/use-session-summary";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const text: Record<string, string> = {
        "agentLocal.sessionSummary.tooltip": "Toggle summary",
        "agentLocal.sessionSummary.title": "Session summary",
        "agentLocal.sessionSummary.environment": "Environment",
        "agentLocal.sessionSummary.modifications": "Changes",
        "agentLocal.sessionSummary.branch": "Branch",
        "agentLocal.sessionSummary.noGit": "No Git repository",
        "agentLocal.sessionSummary.sections.todos": "Todo list",
        "agentLocal.sessionSummary.sections.plans": "Plan",
        "agentLocal.sessionSummary.sections.subagents": "Subagents",
        "common.loading": "Loading",
      };
      void opts;
      return text[key] ?? key;
    },
  }),
}));

vi.mock("@/components/ui/icons", () => ({
  Archive: () => <span />,
  CaretDown: () => <span data-testid="caret" />,
  CheckCircle2: () => <span />,
  ChevronDown: () => <span />,
  Circle: () => <span />,
  Clock3: () => <span />,
  FileText: () => <span data-testid="file-text" />,
  GitBranch: () => <span data-testid="git-branch" />,
  PauseCircle: () => <span />,
  CaretLeft: () => <span data-testid="back" />,
  X: () => <span data-testid="close" />,
}));

vi.mock("@/hooks/use-github-branch-auth", () => ({
  useGithubBranchAuth: () => ({
    open: false,
    state: "idle",
    request: vi.fn(),
    cancel: vi.fn(),
    connect: vi.fn(),
  }),
}));

const git = {
  repositoryPath: "/repo",
  isGitRepo: true,
  isLoading: false,
  currentBranch: "main",
  branches: [{ name: "main", is_current: true, is_remote: false, dirty_count: 0 }],
  worktrees: [],
  dirtyCount: 0,
  uncommittedSnapshotStatus: "ready" as const,
  hasRemote: true,
  remoteStatusError: false,
  isGithubRemote: true,
  hasRemoteBranch: true,
  aheadCount: 0,
  behindCount: 0,
  listDirtyFiles: vi.fn().mockResolvedValue([]),
  listCommits: vi.fn().mockResolvedValue({ commits: [] }),
  listCommitFiles: vi.fn().mockResolvedValue([]),
  commit: vi.fn().mockResolvedValue({ ok: true }),
  push: vi.fn().mockResolvedValue({ ok: true }),
  previewBranchMerge: vi.fn(),
  mergeBranch: vi.fn().mockResolvedValue({ ok: true }),
  refresh: vi.fn(),
};

describe("SessionSummaryBubble", () => {
  it("ouvre la bulle et affiche modifications + branche", () => {
    const { container, getByRole, getByText } = render(
      <SessionSummaryBubble
        summary={summary()}
        git={{
          ...git,
          dirtyCount: 1,
          uncommittedSnapshot: {
            head_commit: "a".repeat(40),
            total_files: 1,
            truncated: false,
            files: [{ path: "src/app.ts", status: "modified", additions: 14, deletions: 6 }],
          },
        }}
      />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));

    expect(getByRole("dialog", { name: "Session summary" })).toBeTruthy();
    expect(getByText("Environment")).toBeTruthy();
    expect(getByText("+14")).toBeTruthy();
    expect(getByText("-6")).toBeTruthy();
    expect(getByText("main")).toBeTruthy();
    expect(container.querySelectorAll(".ssb-section-icon")).toHaveLength(3);
    expect(container.querySelectorAll(".ssb-row-icon[aria-hidden='true']")).toHaveLength(2);
  });

  it("affiche un fallback sans dépôt Git", () => {
    const { getByRole, getByText } = render(
      <SessionSummaryBubble
        summary={summary({ additions: 0, deletions: 0 })}
        git={{ ...git, isGitRepo: false, currentBranch: "" }}
      />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));

    expect(getByText("+0")).toBeTruthy();
    expect(getByText("-0")).toBeTruthy();
    expect(getByText("No Git repository")).toBeTruthy();
  });

  it("ignore l'historique de session quand le worktree Git est propre", () => {
    const { getByRole, getByText, queryByText } = render(
      <SessionSummaryBubble summary={summary({ additions: 9, deletions: 4 })} git={git} />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));

    expect(getByText("+0")).toBeTruthy();
    expect(getByText("-0")).toBeTruthy();
    expect(queryByText("+9")).toBeNull();
    expect(queryByText("-4")).toBeNull();
  });

  it("affiche le fallback branche si git est absent", () => {
    const { getByRole, getByText } = render(
      <SessionSummaryBubble summary={summary()} />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));

    expect(getByText("No Git repository")).toBeTruthy();
  });

  it("ferme la bulle avec Échap sans propager l'événement", () => {
    const { getByRole, queryByRole } = render(
      <SessionSummaryBubble summary={summary()} git={git} />,
    );
    fireEvent.click(getByRole("button", { name: "Toggle summary" }));
    expect(getByRole("dialog", { name: "Session summary" })).toBeTruthy();

    const globalShortcut = vi.fn();
    window.addEventListener("keydown", globalShortcut);
    const event = new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true });
    act(() => {
      window.dispatchEvent(event);
    });
    window.removeEventListener("keydown", globalShortcut);

    expect(event.defaultPrevented).toBe(true);
    expect(globalShortcut).not.toHaveBeenCalled();
    expect(queryByRole("dialog", { name: "Session summary" })).toBeNull();
  });
});

function summary(changes = { additions: 3, deletions: 1 }): SessionSummaryHookState {
  return {
    session: null,
    changes: { ...changes, files: changes.additions + changes.deletions > 0 ? 1 : 0 },
    todoRuns: [
      {
        id: "todo-1",
        title: "Implement UI",
        status: "active",
        created_at: "",
        updated_at: "",
        todos: [
          { content: "One", status: "completed" },
          { content: "Two", status: "pending" },
        ],
      },
      {
        id: "todo-2",
        title: "Paused work",
        status: "paused",
        created_at: "",
        updated_at: "",
        todos: [{ content: "Later", status: "pending" }],
      },
    ],
    plans: [{
      id: "plan-1",
      title: "Plan title",
      status: "awaiting_approval",
      path: "/tmp/plan.md",
      created_at: "",
      updated_at: "",
    }],
    subagents: [{
      sessionId: "child-1",
      name: "Explorer",
      type: "explorer",
      status: "interrupted",
      promptPreview: "",
      description: "Analyse sous-agent",
    }],
  };
}
