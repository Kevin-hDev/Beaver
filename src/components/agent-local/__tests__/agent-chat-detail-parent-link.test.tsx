import { act, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AgentChatDetail } from "../agent-chat-detail";

const harness = vi.hoisted(() => ({ instantLayout: false }));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("../chat-view", () => ({
  ChatView: ({ instantLayout }: { instantLayout?: boolean }) => {
    harness.instantLayout = instantLayout ?? false;
    return null;
  },
}));
vi.mock("@/components/agent-side-panel/agent-side-panel", () => ({ AgentSidePanel: () => null }));
vi.mock("@/components/file-preview/file-preview-panel", () => ({ FilePreviewPanel: () => null }));
vi.mock("@/components/file-tree/file-tree-panel", () => ({ FileTreePanel: () => null }));
vi.mock("@/components/internal-browser/browser-panel", () => ({ BrowserPanel: () => null }));
vi.mock("@/hooks/use-agent-panel-layout", () => ({
  useAgentPanelLayout: () => ({
    containerRef: { current: null },
    layout: { chatMinWidth: 0, previewWidth: 0, fileTreeWidth: 0 },
  }),
}));

describe("AgentChatDetail parent navigation", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    harness.instantLayout = false;
  });

  it("utilise la traduction du retour au chat parent", () => {
    render(<AgentChatDetail {...({
      sessionId: "child",
      parentSessionId: "parent",
      parentSessionName: "Parent",
      filePreview: {
        width: 0, extraWidth: 0, fullscreen: false, open: false, resizing: false,
        tabs: [], activeTab: null, listMode: "latest",
      },
      fileTree: { open: false, width: 0, hasProject: false },
      fileOperations: { all: [], latest: [] },
      gitUncommittedFiles: [],
      projects: [],
      git: {},
      terminal: {},
    } as unknown as Parameters<typeof AgentChatDetail>[0])} />);

    expect(screen.getByRole("button", { name: "← agentLocal.parentChat" })).toBeInTheDocument();
  });

  it("applique la mise en page finale sans animation pendant un changement de session", () => {
    const frames: FrameRequestCallback[] = [];
    vi.stubGlobal("requestAnimationFrame", vi.fn((callback: FrameRequestCallback) => {
      frames.push(callback);
      return frames.length;
    }));
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
    const props = {
      workspaceSessionId: "root-a",
      sessionId: "root-a",
      filePreview: {
        width: 0, extraWidth: 0, fullscreen: false, open: true, resizing: false,
        tabs: [], activeTab: null, listMode: "latest",
      },
      fileTree: { open: false, width: 0, hasProject: false },
      fileOperations: { all: [], latest: [] },
      gitUncommittedFiles: [],
      projects: [],
      git: {},
      terminal: {},
    } as unknown as Parameters<typeof AgentChatDetail>[0];
    const view = render(<AgentChatDetail {...props} />);

    view.rerender(<AgentChatDetail {...props} workspaceSessionId="root-b" sessionId="root-b" />);

    expect(view.container.querySelector(".agent-detail-session-switching")).not.toBeNull();
    expect(harness.instantLayout).toBe(true);

    act(() => { frames.shift()?.(0); });
    act(() => { frames.shift()?.(16); });

    expect(view.container.querySelector(".agent-detail-session-switching")).toBeNull();
    expect(harness.instantLayout).toBe(false);
  });
});
