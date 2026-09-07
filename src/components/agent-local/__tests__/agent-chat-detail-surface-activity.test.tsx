import { render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { AgentChatDetail } from "../agent-chat-detail";

const harness = vi.hoisted(() => ({ browserActive: null as boolean | null }));

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("../chat-view", () => ({ ChatView: () => null }));
vi.mock("@/components/agent-side-panel/agent-side-panel", () => ({
  AgentSidePanel: ({ browserContent }: { browserContent: React.ReactNode }) => browserContent,
}));
vi.mock("@/components/file-preview/file-preview-panel", () => ({ FilePreviewPanel: () => null }));
vi.mock("@/components/file-tree/file-tree-panel", () => ({ FileTreePanel: () => null }));
vi.mock("@/components/internal-browser/browser-panel", () => ({
  BrowserPanel: ({ active }: { active: boolean }) => {
    harness.browserActive = active;
    return null;
  },
}));
vi.mock("@/hooks/use-agent-panel-layout", () => ({
  useAgentPanelLayout: () => ({
    containerRef: { current: null },
    layout: { chatMinWidth: 0, previewWidth: 0, fileTreeWidth: 0 },
  }),
}));

const props = {
  workspaceSessionId: "root-a",
  sessionId: "root-a",
  panelMode: "browser",
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

describe("AgentChatDetail et l'activité de surface", () => {
  afterEach(() => {
    harness.browserActive = null;
  });

  it("désactive puis réactive la WebView sans démonter le détail", () => {
    const view = render(
      <AppSurfaceActivityProvider active>
        <AgentChatDetail {...props} />
      </AppSurfaceActivityProvider>,
    );
    expect(harness.browserActive).toBe(true);

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <AgentChatDetail {...props} />
      </AppSurfaceActivityProvider>,
    );
    expect(harness.browserActive).toBe(false);

    view.rerender(
      <AppSurfaceActivityProvider active>
        <AgentChatDetail {...props} />
      </AppSurfaceActivityProvider>,
    );
    expect(harness.browserActive).toBe(true);
  });
});
