import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act, render } from "@testing-library/react";
import { TerminalPanel } from "../terminal-panel";
import type { TerminalTab } from "@/hooks/use-terminal";

/* Refermer le panneau démontait les écrans, et le démontage tuait les shells :
   un serveur lancé dans un onglet mourait avec lui, et tout ce que les
   terminaux contenaient disparaissait. Rien ne l'annonçait — il fallait garder
   le panneau ouvert en permanence pour ne rien perdre.
   Les shells vivent maintenant tant que l'application vit ; seule la croix d'un
   onglet, ou la fermeture de l'application, les arrête. */

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
    onResize() {}
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
  cwd: "/projet",
  hasActivity: false,
};

function panel(isOpen: boolean) {
  return (
    <TerminalPanel
      tabs={[TAB]}
      activeTabId={TAB.id}
      allTabs={[{ tab: TAB, groupKey: "projet" }]}
      activeGroupKey="projet"
      isOpen={isOpen}
      panelHeight={200}
      onAddTab={vi.fn()}
      onCloseTab={vi.fn()}
      onSelectTab={vi.fn()}
      onRenameTab={vi.fn()}
      onReorderTabs={vi.fn()}
      onTogglePanel={vi.fn()}
      onPtyReady={vi.fn()}
      onTabActivity={vi.fn()}
      onResize={vi.fn()}
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
  it("ne tue aucun shell quand on referme le panneau", () => {
    const { rerender } = render(panel(true));

    rerender(panel(false));
    act(() => {
      vi.advanceTimersByTime(WELL_AFTER_CLOSING_MS);
    });

    expect(killCalls()).toEqual([]);
  });

  /* Le shell doit rester joignable : c'est ce qui distingue « caché » de
     « arrêté ». */
  it("garde l'écran monté après la fermeture", () => {
    const { container, rerender } = render(panel(true));

    rerender(panel(false));
    act(() => {
      vi.advanceTimersByTime(WELL_AFTER_CLOSING_MS);
    });

    expect(screenOf(container)).not.toBeNull();
  });

  /* Caché ne veut pas dire actif : un écran invisible qui garderait le focus
     avalerait les touches frappées dans la conversation. */
  it("rend l'écran inactif tant que le panneau est fermé", () => {
    const { container, rerender } = render(panel(true));

    rerender(panel(false));
    act(() => {
      vi.advanceTimersByTime(WELL_AFTER_CLOSING_MS);
    });

    expect(screenOf(container)?.style.visibility).toBe("hidden");
  });

  it("le rend de nouveau actif à la réouverture", () => {
    const { container, rerender } = render(panel(true));
    rerender(panel(false));
    act(() => {
      vi.advanceTimersByTime(WELL_AFTER_CLOSING_MS);
    });

    rerender(panel(true));

    expect(screenOf(container)?.style.visibility).toBe("visible");
  });

  it("ne monte rien tant que le panneau n'a jamais été ouvert", () => {
    const { container } = render(panel(false));

    expect(screenOf(container)).toBeNull();
    expect(invoke).not.toHaveBeenCalled();
  });
});
