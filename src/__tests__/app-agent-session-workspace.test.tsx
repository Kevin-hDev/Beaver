import { fireEvent, render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "@/App";
import { useArrowNavigation } from "@/hooks/use-arrow-navigation";
import type { AgentLocalNavState, AgentLocalWorkspaceState } from "@/types/navigation";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@/hooks/use-theme", () => ({
  useTheme: () => ({ choice: "dark", setTheme: vi.fn(), setThemeCatalog: vi.fn() }),
}));
vi.mock("@/hooks/use-startup-gate", () => ({
  useStartupGate: () => ({ view: "ready" }),
}));
vi.mock("@/hooks/use-arrow-navigation", () => ({ useArrowNavigation: vi.fn() }));
vi.mock("@/hooks/use-panel-focus", () => ({ usePanelFocus: () => ({ focusedPanel: "detail" }) }));
vi.mock("@/hooks/use-platform-body-class", () => ({ usePlatformBodyClass: vi.fn() }));
vi.mock("@/hooks/use-browser-recovery-notice", () => ({ useBrowserRecoveryNotice: vi.fn() }));
vi.mock("@/hooks/use-extensions", () => ({
  ExtensionsProvider: ({ children }: { children: ReactNode }) => children,
  useExtensions: () => ({ extensions: [] }),
}));
vi.mock("@/hooks/update-context", () => ({ UpdateProvider: ({ children }: { children: ReactNode }) => children }));
vi.mock("@/hooks/use-app-navigation-actions", () => ({
  AppNavigationActionsProvider: ({ children }: { children: ReactNode }) => children,
}));
vi.mock("@/components/layout/app-layout", () => ({
  AppLayout: ({
    children,
    onBack,
    onForward,
  }: {
    children: ReactNode;
    onBack: () => void;
    onForward: () => void;
  }) => (
    <>
      <button type="button" onClick={onBack}>Retour</button>
      <button type="button" onClick={onForward}>Suivant</button>
      {children}
    </>
  ),
}));
vi.mock("@/components/heartbeat/heartbeat-tab", () => ({ HeartbeatTab: () => null }));
vi.mock("@/components/personality/personality-tab", () => ({ PersonalityTab: () => null }));
vi.mock("@/components/settings/settings-tab", () => ({ SettingsTab: () => null }));

vi.mock("@/components/agent-local/agent-local-tab", () => ({
  AgentLocalTab: ({
    navState,
    onSessionChange,
    onNavChange,
  }: {
    navState: AgentLocalNavState;
    onSessionChange: (id: string) => void;
    onNavChange: (partial: Partial<AgentLocalWorkspaceState>) => void;
  }) => (
    <>
      <button type="button" onClick={() => onSessionChange("session-a")}>Session A</button>
      <button type="button" onClick={() => onSessionChange("session-b")}>Session B</button>
      <button type="button" onClick={() => onNavChange({ terminalOpen: !navState.terminalOpen })}>
        Terminal
      </button>
      <button type="button" onClick={() => onNavChange({ previewOpen: !navState.previewOpen })}>
        Panneau
      </button>
      <output aria-label="session active">{navState.sessionId ?? "none"}</output>
      <output aria-label="terminal ouvert">{String(navState.terminalOpen)}</output>
      <output aria-label="panneau ouvert">{String(navState.previewOpen)}</output>
    </>
  ),
}));

describe("état visuel des sessions Agent Local", () => {
  beforeEach(() => {
    window.location.hash = "";
  });

  it("ouvre B avec son état fermé puis restaure exactement A", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Session A" }));
    fireEvent.click(screen.getByRole("button", { name: "Terminal" }));
    fireEvent.click(screen.getByRole("button", { name: "Panneau" }));
    expect(screen.getByRole("status", { name: "terminal ouvert" })).toHaveTextContent("true");
    expect(screen.getByRole("status", { name: "panneau ouvert" })).toHaveTextContent("true");

    fireEvent.click(screen.getByRole("button", { name: "Session B" }));
    expect(screen.getByRole("status", { name: "session active" })).toHaveTextContent("session-b");
    expect(screen.getByRole("status", { name: "terminal ouvert" })).toHaveTextContent("false");
    expect(screen.getByRole("status", { name: "panneau ouvert" })).toHaveTextContent("false");

    fireEvent.click(screen.getByRole("button", { name: "Session A" }));
    expect(screen.getByRole("status", { name: "terminal ouvert" })).toHaveTextContent("true");
    expect(screen.getByRole("status", { name: "panneau ouvert" })).toHaveTextContent("true");
  });

  it("donne à la navigation clavier l'ordre résolu du registre", () => {
    render(<App />);

    expect(vi.mocked(useArrowNavigation)).toHaveBeenCalledWith(expect.objectContaining({
      items: ["agent-local", "heartbeat", "personality", "settings"],
    }));
  });

  it("conserve les états par session pendant retour et suivant", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Session A" }));
    fireEvent.click(screen.getByRole("button", { name: "Terminal" }));
    fireEvent.click(screen.getByRole("button", { name: "Session B" }));

    fireEvent.click(screen.getByRole("button", { name: "Retour" }));
    expect(screen.getByRole("status", { name: "session active" })).toHaveTextContent("session-a");
    expect(screen.getByRole("status", { name: "terminal ouvert" })).toHaveTextContent("true");

    fireEvent.click(screen.getByRole("button", { name: "Suivant" }));
    expect(screen.getByRole("status", { name: "session active" })).toHaveTextContent("session-b");
    expect(screen.getByRole("status", { name: "terminal ouvert" })).toHaveTextContent("false");
  });
});
