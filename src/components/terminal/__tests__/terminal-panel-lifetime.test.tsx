import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { TerminalPanel } from "../terminal-panel";
import type { TerminalTab } from "@/hooks/use-terminal";

/* Le repli ne démonte plus les écrans : seuls la croix d'un onglet ou l'arrêt
   de l'application mettent fin aux shells. */

const invoke = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
  Channel: class {
    onmessage: ((event: unknown) => void) | null = null;
  },
}));

vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    cols = 80;
    rows = 24;
    options: Record<string, unknown> = {};
    loadAddon() {}
    open() {}
    write() {}
    writeln() {}
    focus() {}
    dispose() {}
    onData() {}
    onResize() {
      return { dispose() {} };
    }
    attachCustomKeyEventHandler() {}
    getSelection() {
      return "";
    }
  },
}));

vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit() {}
  },
}));

vi.mock("@xterm/xterm/css/xterm.css", () => ({}));

/* Au-delà de tout délai que la fermeture pourrait s'accorder. */
const WELL_AFTER_CLOSING_MS = 5000;

const TAB: TerminalTab = {
  id: "serveur",
  ptyId: null,
  ptyToken: null,
  label: "serveur",
  hasActivity: false,
};

const OTHER_TAB: TerminalTab = {
  id: "shell",
  ptyId: null,
  ptyToken: null,
  label: "shell",
  hasActivity: false,
};

function panel(
  isOpen: boolean,
  allTabs = [{ tab: TAB, groupKey: "projet" }],
  activeGroupKey = "projet",
  activeTab = TAB,
  onCloseTab = vi.fn(),
  panelHeight = 200,
  onResize = vi.fn(),
) {
  return (
    <TerminalPanel
      tabs={[activeTab]}
      activeTabId={activeTab.id}
      allTabs={allTabs}
      activeGroupKey={activeGroupKey}
      isOpen={isOpen}
      panelHeight={panelHeight}
      onAddTab={vi.fn()}
      onCloseTab={onCloseTab}
      onSelectTab={vi.fn()}
      onRenameTab={vi.fn()}
      onReorderTabs={vi.fn()}
      onTogglePanel={vi.fn()}
      onPtyReady={vi.fn()}
      onTabActivity={vi.fn()}
      onProcessExit={vi.fn()}
      onLiveLimitReached={vi.fn()}
      onResize={onResize}
      onSetMaxHeight={vi.fn()}
    />
  );
}

function killCalls(): unknown[][] {
  return invoke.mock.calls.filter((call) => call[0] === "pty_kill");
}

function screenOf(container: HTMLElement): HTMLElement | null {
  return container.querySelector(".terminal-screen");
}

function finishClosing(rerender: ReturnType<typeof render>["rerender"]) {
  rerender(panel(false));
  act(() => { vi.advanceTimersByTime(WELL_AFTER_CLOSING_MS); });
}

beforeEach(() => {
  vi.useFakeTimers();
  invoke.mockReset();
  invoke.mockResolvedValue({ id: 7, token: "jeton" });
  /* jsdom n'observe pas les tailles ; l'écran en construit un au montage. */
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      disconnect() {}
      unobserve() {}
    },
  );
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

describe("durée de vie des shells du panneau", () => {
  it("transmet la clé de groupe à chaque terminal rendu", () => {
    const allTabs = [
      { tab: TAB, groupKey: "projet" },
      { tab: OTHER_TAB, groupKey: "__default__" },
    ];
    const { rerender } = render(panel(true, allTabs));
    rerender(panel(true, allTabs, "__default__", OTHER_TAB));

    const spawnPayloads = invoke.mock.calls
      .filter((call) => call[0] === "pty_spawn")
      .map((call): unknown => call[1] as unknown);
    expect(spawnPayloads).toEqual([
      expect.objectContaining({ groupKey: "projet" }),
      expect.objectContaining({ groupKey: "__default__" }),
    ]);
  });

  it("n'envoie jamais cwd dans la charge utile pty_spawn", () => {
    const source = readFileSync(
      "src/components/terminal/terminal-pty-bridge.ts",
      "utf8",
    );
    const payload = source.match(
      /invoke<[^>]+>\("pty_spawn",\s*\{([\s\S]*?)\}\)/,
    )?.[1];

    expect(payload).toBeDefined();
    expect(payload).toContain("groupKey");
    expect(payload).not.toMatch(/\bcwd\s*:/);
  });

  it("ne tue aucun shell quand on referme le panneau", () => {
    const { rerender } = render(panel(true));
    finishClosing(rerender);

    expect(killCalls()).toEqual([]);
  });

  it("ferme immédiatement un onglet qui n'a pas encore de PTY", () => {
    const onCloseTab = vi.fn();
    const { container } = render(panel(true, undefined, undefined, undefined, onCloseTab));

    fireEvent.click(container.querySelector(".terminal-tab-close")!);

    expect(killCalls()).toEqual([]);
    expect(onCloseTab).toHaveBeenCalledWith(TAB.id);
  });

  /* Le shell doit rester joignable : c'est ce qui distingue « caché » de
     « arrêté ». */
  it("garde l'écran monté après la fermeture", () => {
    const { container, rerender } = render(panel(true));
    finishClosing(rerender);

    expect(screenOf(container)).not.toBeNull();
  });

  /* Caché ne veut pas dire actif : un écran invisible qui garderait le focus
     avalerait les touches frappées dans la conversation. */
  it("rend l'écran inactif tant que le panneau est fermé", () => {
    const { container, rerender } = render(panel(true));
    finishClosing(rerender);

    expect(screenOf(container)?.style.visibility).toBe("hidden");
  });

  it("le rend de nouveau actif à la réouverture", () => {
    const { container, rerender } = render(panel(true));
    finishClosing(rerender);
    rerender(panel(true));

    expect(screenOf(container)?.style.visibility).toBe("visible");
  });

  it("affiche exactement la hauteur bornée retournée pendant le drag", () => {
    const onResize = vi.fn((requested: number) => requested > 400 ? 400 : 80);
    const { container } = render(panel(true, undefined, undefined, undefined, undefined, 120, onResize));
    const handle = container.querySelector(".terminal-resize-handle")!;
    const renderedPanel = container.querySelector(".terminal-panel") as HTMLElement;
    fireEvent.pointerDown(handle, { clientY: 0 });
    for (const movement of [
      { clientY: -780, requested: 900, clamped: 400 },
      { clientY: 100, requested: 20, clamped: 80 },
    ]) {
      fireEvent.pointerMove(window, { clientY: movement.clientY });
      expect(onResize).toHaveBeenLastCalledWith(movement.requested);
      expect(renderedPanel.style.height).toBe(`${movement.clamped}px`);
    }
  });

  it("ne monte rien tant que le panneau n'a jamais été ouvert", () => {
    const { container } = render(panel(false));

    expect(screenOf(container)).toBeNull();
    expect(invoke).not.toHaveBeenCalled();
  });
});
