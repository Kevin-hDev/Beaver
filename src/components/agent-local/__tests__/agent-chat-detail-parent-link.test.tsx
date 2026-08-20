import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AgentChatDetail } from "../agent-chat-detail";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("../chat-view", () => ({ ChatView: () => null }));
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
});
