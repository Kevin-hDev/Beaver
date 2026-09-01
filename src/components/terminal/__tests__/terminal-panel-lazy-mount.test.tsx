import { fireEvent, render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { TerminalPanel } from "../terminal-panel";
import type { TerminalTab } from "@/hooks/use-terminal";

interface InstanceProps {
  tabId: string;
  onExit: (tabId: string) => void;
}

const harness = vi.hoisted(() => ({
  invoke: vi.fn(),
  showToast: vi.fn(),
  mounted: [] as string[],
  unmounted: [] as string[],
  instances: new Map<string, InstanceProps>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: harness.invoke }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: harness.showToast }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("../terminal-instance", async () => {
  const { useEffect } = await import("react");
  return {
    TerminalInstance: (props: InstanceProps) => {
      useEffect(() => {
        harness.mounted.push(props.tabId);
        return () => {
          harness.unmounted.push(props.tabId);
          harness.instances.delete(props.tabId);
        };
      }, [props.tabId]);
      useEffect(() => {
        harness.instances.set(props.tabId, props);
      }, [props]);
      return <div data-testid={`instance-${props.tabId}`} />;
    },
  };
});

const tab = (id: string, ptyId: number | null = null): TerminalTab => ({
  id,
  ptyId,
  ptyToken: ptyId == null ? null : `token-${id}`,
  label: id,
  hasActivity: false,
});

const entry = (id: string, groupKey = "project-a") => ({
  tab: tab(id),
  groupKey,
});

function panelProps(overrides: Record<string, unknown> = {}) {
  const active = tab("a1");
  return {
    tabs: [active],
    activeTabId: active.id,
    allTabs: [{ tab: active, groupKey: "project-a" }],
    activeGroupKey: "project-a",
    isOpen: true,
    panelHeight: 200,
    onAddTab: vi.fn(),
    onCloseTab: vi.fn(),
    onSelectTab: vi.fn(),
    onRenameTab: vi.fn(),
    onReorderTabs: vi.fn(),
    onTogglePanel: vi.fn(),
    onPtyReady: vi.fn(),
    onTabActivity: vi.fn(),
    onProcessExit: vi.fn(),
    onLiveLimitReached: vi.fn(),
    onResize: vi.fn(),
    onSetMaxHeight: vi.fn(),
    ...overrides,
  };
}

function renderPanel(overrides: Record<string, unknown> = {}) {
  let props = panelProps(overrides);
  const view = render(<TerminalPanel {...props} />);
  return {
    ...view,
    props,
    rerenderPanel(next: Record<string, unknown>) {
      props = { ...props, ...next };
      view.rerender(<TerminalPanel {...props} />);
    },
  };
}

beforeEach(() => {
  harness.invoke.mockReset();
  harness.showToast.mockReset();
  harness.mounted.length = 0;
  harness.unmounted.length = 0;
  harness.instances.clear();
});

describe("montage différé des terminaux", () => {
  it("ne monte aucun terminal tant que le panneau reste fermé", () => {
    renderPanel({ isOpen: false });
    expect(harness.mounted).toEqual([]);
  });

  it("monte seulement l'onglet actif puis garde les onglets déjà démarrés", () => {
    const allTabs = [entry("a1"), entry("a2"), entry("b1", "project-b")];
    const view = renderPanel({ allTabs });
    expect(harness.mounted).toEqual(["a1"]);

    view.rerenderPanel({ tabs: [tab("a1"), tab("a2")], activeTabId: "a2" });
    expect(harness.mounted).toEqual(["a1", "a2"]);

    view.rerenderPanel({
      tabs: [tab("b1")],
      activeTabId: "b1",
      activeGroupKey: "project-b",
    });
    expect(harness.mounted).toEqual(["a1", "a2", "b1"]);

    view.rerenderPanel({ isOpen: false });
    expect(harness.unmounted).toEqual([]);
  });

  it("démonte un terminal seulement quand son onglet disparaît", async () => {
    const view = renderPanel({ allTabs: [entry("a1"), entry("a2")] });
    view.rerenderPanel({ tabs: [tab("a2")], activeTabId: "a2" });
    view.rerenderPanel({ allTabs: [entry("a2")] });

    await waitFor(() => expect(harness.unmounted).toContain("a1"));
    expect(harness.instances.has("a2")).toBe(true);
  });

  it("ne monte pas les onglets restaurés qui n'ont jamais été activés", () => {
    const allTabs = Array.from({ length: 20 }, (_, index) => entry(`a${index + 1}`));
    renderPanel({ allTabs });
    expect(harness.mounted).toEqual(["a1"]);
  });

  it("borne les terminaux vivants et réessaie après libération d'une place", async () => {
    const allTabs = Array.from({ length: 17 }, (_, index) => entry(`a${index + 1}`));
    const view = renderPanel({ allTabs });

    for (let index = 2; index <= 16; index += 1) {
      view.rerenderPanel({ activeTabId: `a${index}` });
    }
    expect(harness.mounted).toHaveLength(16);

    view.rerenderPanel({ activeTabId: "a17" });
    view.rerenderPanel({ activeTabId: "a17" });
    expect(harness.mounted).toHaveLength(16);
    expect(view.props.onLiveLimitReached).toHaveBeenCalledWith("a17");

    view.rerenderPanel({ allTabs: allTabs.slice(1) });
    await waitFor(() => expect(harness.mounted).toContain("a17"));
    expect(view.props.onLiveLimitReached).toHaveBeenCalledTimes(1);
  });

  it("associe une fin de processus au groupe de l'onglet démarré", () => {
    const onProcessExit = vi.fn();
    renderPanel({
      tabs: [tab("b1")],
      activeTabId: "b1",
      allTabs: [entry("b1", "project-b")],
      activeGroupKey: "project-b",
      onProcessExit,
    });

    harness.instances.get("b1")?.onExit("b1");
    expect(onProcessExit).toHaveBeenCalledWith("b1", "project-b");
  });
});

describe("fermeture sûre d'un onglet avec PTY", () => {
  it("attend la réussite de pty_kill avant de fermer l'onglet", async () => {
    let finish!: () => void;
    harness.invoke.mockReturnValueOnce(new Promise<void>((resolve) => { finish = resolve; }));
    const running = tab("a1", 9);
    const onCloseTab = vi.fn();
    const { container } = renderPanel({
      tabs: [running],
      allTabs: [{ tab: running, groupKey: "project-a" }],
      onCloseTab,
    });

    fireEvent.click(container.querySelector(".terminal-tab-close")!);
    expect(onCloseTab).not.toHaveBeenCalled();
    finish();
    await waitFor(() => expect(onCloseTab).toHaveBeenCalledWith("a1"));
  });

  it("ferme aussi après la réponse publique terminal-not-found", async () => {
    harness.invoke.mockRejectedValueOnce("terminal-not-found");
    const running = tab("a1", 9);
    const onCloseTab = vi.fn();
    const { container } = renderPanel({
      tabs: [running],
      allTabs: [{ tab: running, groupKey: "project-a" }],
      onCloseTab,
    });

    fireEvent.click(container.querySelector(".terminal-tab-close")!);
    await waitFor(() => expect(onCloseTab).toHaveBeenCalledWith("a1"));
  });

  it("conserve l'onglet et affiche une erreur générique si pty_kill échoue", async () => {
    harness.invoke.mockRejectedValueOnce("internal-detail");
    const running = tab("a1", 9);
    const onCloseTab = vi.fn();
    const { container } = renderPanel({
      tabs: [running],
      allTabs: [{ tab: running, groupKey: "project-a" }],
      onCloseTab,
    });

    fireEvent.click(container.querySelector(".terminal-tab-close")!);
    await waitFor(() => {
      expect(harness.showToast).toHaveBeenCalledWith("terminal.failedToClose", "error");
    });
    expect(onCloseTab).not.toHaveBeenCalled();
  });
});
