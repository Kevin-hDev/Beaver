import { useState } from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useArrowNavigation } from "../use-arrow-navigation";
import { usePanelFocus } from "../use-panel-focus";

const TABS = ["agent-local", "heartbeat"] as const;
const SESSIONS = ["s1", "s2"] as const;

function KeyboardHarness() {
  const { focusedPanel } = usePanelFocus();
  const [tab, setTab] = useState<(typeof TABS)[number]>("agent-local");
  const [session, setSession] = useState<(typeof SESSIONS)[number]>("s1");

  useArrowNavigation({
    items: [...TABS],
    selectedId: tab,
    onSelect: setTab,
    enabled: focusedPanel === "sidebar",
    focusActiveSelector: "[data-nav-zone='sidebar'] [data-nav-active='true']",
  });

  useArrowNavigation({
    items: [...SESSIONS],
    selectedId: session,
    onSelect: setSession,
    enabled: focusedPanel === "list",
    focusActiveSelector: "[data-nav-zone='list'] [data-nav-active='true']",
  });

  return (
    <>
      <nav data-nav-zone="sidebar" tabIndex={-1}>
        {TABS.map((id) => (
          <button
            key={id}
            data-testid={`tab-${id}`}
            data-nav-active={tab === id ? "true" : undefined}
            tabIndex={tab === id ? 0 : -1}
            onClick={() => setTab(id)}
          >
            {id}
          </button>
        ))}
      </nav>
      <section data-nav-zone="list" tabIndex={-1}>
        <input data-testid="list-checkbox" type="checkbox" readOnly />
        {SESSIONS.map((id) => (
          <button
            key={id}
            data-testid={`session-${id}`}
            data-nav-active={session === id ? "true" : undefined}
            tabIndex={session === id ? 0 : -1}
            onClick={() => setSession(id)}
          >
            {id}
          </button>
        ))}
        <textarea data-testid="chat-input" />
      </section>
      <section data-nav-zone="detail" data-testid="active-detail" tabIndex={-1}>
        <button data-testid="detail-button">detail</button>
      </section>
      <div role="menu">
        <button data-testid="menu-item">menu</button>
      </div>
      <button className="xterm" data-testid="terminal" type="button" />
      <span data-testid="state">{tab}:{session}</span>
    </>
  );
}

function EmptyListHarness() {
  const { focusedPanel } = usePanelFocus();

  return (
    <>
      <nav data-nav-zone="sidebar" tabIndex={-1}>
        <button data-testid="tab-heartbeat" data-nav-active="true">heartbeat</button>
      </nav>
      <section data-nav-zone="list" tabIndex={-1} data-testid="empty-list">
        <input data-testid="empty-checkbox" type="checkbox" readOnly />
        <span>No wakeups</span>
      </section>
      <section data-nav-zone="detail" tabIndex={-1}>
        <button data-testid="detail-button">detail</button>
      </section>
      <span data-testid="focused-panel">{focusedPanel}</span>
    </>
  );
}

function InactiveTerminalHarness() {
  usePanelFocus();
  return (
    <>
      <section data-nav-zone="detail" data-testid="active-detail" tabIndex={-1}>
        <button data-testid="active-detail-button">detail</button>
      </section>
      <div hidden inert={true} data-testid="inactive-surface">
        <section data-nav-zone="terminal" tabIndex={-1}>
          <button data-testid="hidden-terminal">terminal</button>
        </section>
      </div>
    </>
  );
}

function DuplicateTerminalHarness() {
  usePanelFocus();
  return (
    <>
      <section data-nav-zone="detail" tabIndex={-1}>
        <button data-testid="duplicate-detail">detail</button>
      </section>
      <div hidden inert={true}>
        <section data-nav-zone="terminal" tabIndex={-1}>
          <button data-testid="hidden-terminal-first">terminal caché</button>
        </section>
      </div>
      <section data-nav-zone="terminal" tabIndex={-1}>
        <button data-testid="visible-terminal-second">terminal visible</button>
      </section>
    </>
  );
}

function NestedTerminalHarness() {
  usePanelFocus();
  return (
    <section data-nav-zone="terminal" tabIndex={-1}>
      <div hidden inert={true}>
        <button data-testid="nested-hidden-terminal">terminal caché</button>
      </div>
      <button data-testid="nested-visible-terminal">terminal visible</button>
    </section>
  );
}

describe("keyboard navigation", () => {
  afterEach(() => {
    cleanup();
    vi.useRealTimers();
  });

  beforeEach(() => {
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((cb) => {
      window.setTimeout(() => cb(0), 0);
      return 0;
    });
  });

  it("déplace le vrai focus entre sidebar et liste", async () => {
    render(<KeyboardHarness />);
    const tab = screen.getByTestId("tab-agent-local");

    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowRight" });

    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("session-s1")));

    fireEvent.keyDown(screen.getByTestId("session-s1"), { key: "ArrowLeft" });

    await waitFor(() => expect(document.activeElement).toBe(tab));
  });

  it("ne piège pas la navigation quand la liste commence par un input", async () => {
    render(<EmptyListHarness />);
    const tab = screen.getByTestId("tab-heartbeat");

    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowRight" });

    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("empty-list")));

    fireEvent.keyDown(screen.getByTestId("empty-list"), { key: "ArrowLeft" });

    await waitFor(() => expect(document.activeElement).toBe(tab));
    expect(screen.getByTestId("focused-panel").textContent).toBe("sidebar");
  });

  it("navigue haut/bas dans la zone active et garde le focus synchronisé", async () => {
    render(<KeyboardHarness />);
    const first = screen.getByTestId("session-s1");

    first.focus();
    fireEvent.keyDown(first, { key: "ArrowDown" });

    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("session-s2")));
    expect(screen.getByTestId("state").textContent).toBe("agent-local:s2");
  });

  it("ignore les champs texte, menus locaux et terminal", () => {
    render(<KeyboardHarness />);

    screen.getByTestId("chat-input").focus();
    fireEvent.keyDown(screen.getByTestId("chat-input"), { key: "ArrowDown" });
    expect(screen.getByTestId("state").textContent).toBe("agent-local:s1");

    screen.getByTestId("menu-item").focus();
    fireEvent.keyDown(screen.getByTestId("menu-item"), { key: "ArrowDown" });
    expect(screen.getByTestId("state").textContent).toBe("agent-local:s1");

    screen.getByTestId("terminal").focus();
    fireEvent.keyDown(screen.getByTestId("terminal"), { key: "ArrowDown" });
    expect(screen.getByTestId("state").textContent).toBe("agent-local:s1");
  });

  it("ne sélectionne pas un terminal sous une surface inactive", async () => {
    render(<InactiveTerminalHarness />);
    const detail = screen.getByTestId("active-detail-button");
    detail.focus();
    fireEvent.keyDown(detail, { key: "ArrowRight" });

    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("active-detail-button")));
    expect(document.activeElement).not.toBe(screen.getByTestId("hidden-terminal"));
  });

  it("ignore le premier terminal caché et focalise le terminal visible suivant", async () => {
    render(<DuplicateTerminalHarness />);
    const detail = screen.getByTestId("duplicate-detail");
    act(() => {
      detail.focus();
      fireEvent.keyDown(detail, { key: "ArrowRight" });
    });

    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("visible-terminal-second")));
    expect(document.activeElement).not.toBe(screen.getByTestId("hidden-terminal-first"));
  });

  it("filtre aussi les descendants cachés dans la zone active", () => {
    vi.useFakeTimers();
    render(<NestedTerminalHarness />);
    const visibleSource = screen.getByTestId("nested-visible-terminal");
    act(() => {
      visibleSource.focus();
      fireEvent.keyDown(visibleSource, { key: "ArrowLeft" });
      vi.runAllTimers();
    });

    expect(document.activeElement).toBe(screen.getByTestId("nested-visible-terminal"));
    expect(document.activeElement).not.toBe(screen.getByTestId("nested-hidden-terminal"));
  });
});
